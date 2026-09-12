Unicode true
!include "MUI2.nsh"

!ifndef APP_VERSION
  !define APP_VERSION "0.2.0"
!endif
; VIProductVersion demands exactly four numeric fields. build.ps1 derives this
; from the crate version; the default keeps a bare makensis run working.
!ifndef APP_VERSION_4
  !define APP_VERSION_4 "0.2.0.0"
!endif
; MakeNSIS changes its working directory to this script's directory by default.
; Do not prepend __FILEDIR__ again: it can itself be relative to the caller.
; The build helper and CI may override these defaults with absolute paths.
!ifndef APP_BINARY
  !define APP_BINARY "..\..\target\release\system-designer.exe"
!endif
!ifndef OUT_FILE
  !define OUT_FILE "..\..\dist\system-designer-setup.exe"
!endif

!ifndef APP_ICON
  !define APP_ICON "..\..\assets\branding\windows\system-designer.ico"
!endif
!define MUI_ICON "${APP_ICON}"
!define MUI_UNICON "${APP_ICON}"

!define ARP "Software\Microsoft\Windows\CurrentVersion\Uninstall\SystemDesigner"

Name "System Designer"
OutFile "${OUT_FILE}"
InstallDir "$LOCALAPPDATA\Programs\System Designer"
InstallDirRegKey HKCU "Software\System Designer" "InstallDir" "$INSTDIR"
; Per-user install: no administrator rights required, nothing written outside
; the user's own profile and HKCU.
RequestExecutionLevel user
SetCompressor /SOLID lzma

VIProductVersion "${APP_VERSION_4}"
VIAddVersionKey "ProductName" "System Designer"
VIAddVersionKey "FileDescription" "System Designer installer"
VIAddVersionKey "FileVersion" "${APP_VERSION}"
VIAddVersionKey "ProductVersion" "${APP_VERSION}"
VIAddVersionKey "LegalCopyright" "MIT licensed"

!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\SystemDesigner.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Launch System Designer"
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "System Designer" SEC_APP
  SectionIn RO
  SetShellVarContext current
  SetOutPath "$INSTDIR"
  File /oname=SystemDesigner.exe "${APP_BINARY}"
  WriteRegStr HKCU "Software\System Designer" "InstallDir" "$INSTDIR"
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  CreateDirectory "$SMPROGRAMS\System Designer"
  CreateShortcut "$SMPROGRAMS\System Designer\System Designer.lnk" "$INSTDIR\SystemDesigner.exe"

  WriteRegStr HKCU "${ARP}" "DisplayName" "System Designer"
  WriteRegStr HKCU "${ARP}" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKCU "${ARP}" "DisplayIcon" '"$INSTDIR\SystemDesigner.exe",0'
  WriteRegStr HKCU "${ARP}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${ARP}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegStr HKCU "${ARP}" "QuietUninstallString" '"$INSTDIR\Uninstall.exe" /S'
  WriteRegDWORD HKCU "${ARP}" "NoModify" 1
  WriteRegDWORD HKCU "${ARP}" "NoRepair" 1
SectionEnd

Section "Desktop shortcut" SEC_DESKTOP
  SetShellVarContext current
  CreateShortcut "$DESKTOP\System Designer.lnk" "$INSTDIR\SystemDesigner.exe"
SectionEnd

LangString DESC_APP ${LANG_ENGLISH} "The System Designer application and its Start Menu entry."
LangString DESC_DESKTOP ${LANG_ENGLISH} "Also place a System Designer shortcut on the desktop."
!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SEC_APP} $(DESC_APP)
  !insertmacro MUI_DESCRIPTION_TEXT ${SEC_DESKTOP} $(DESC_DESKTOP)
!insertmacro MUI_FUNCTION_DESCRIPTION_END

Section "Uninstall"
  SetShellVarContext current
  Delete "$DESKTOP\System Designer.lnk"
  Delete "$SMPROGRAMS\System Designer\System Designer.lnk"
  RMDir "$SMPROGRAMS\System Designer"
  Delete "$INSTDIR\SystemDesigner.exe"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"
  DeleteRegKey HKCU "${ARP}"
  DeleteRegKey HKCU "Software\System Designer"
  ; Deliberately preserve project files, backups, user data, and recovery copies.
SectionEnd

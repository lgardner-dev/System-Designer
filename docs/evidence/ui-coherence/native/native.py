import os,time,json
from pathlib import Path
from Xlib import display,X,XK
from Xlib.ext import xtest
from PIL import Image
D=display.Display()
def window():
 for w in D.screen().root.query_tree().children:
  if w.get_wm_name()==os.environ.get('SD_NATIVE_WINDOW','System Designer'):return w
 raise RuntimeError('ordinary application window missing')
def click(x,y,button=1):
 w=window();p=D.screen().root.translate_coords(w,0,0)
 xtest.fake_input(D,X.MotionNotify,x=int(p.x+x),y=int(p.y+y));D.sync()
 xtest.fake_input(D,X.ButtonPress,button);D.sync();time.sleep(.08)
 xtest.fake_input(D,X.ButtonRelease,button);D.sync();time.sleep(.25)
def key(name,mod=None):
 if mod:xtest.fake_input(D,X.KeyPress,D.keysym_to_keycode(XK.string_to_keysym(mod)))
 k=D.keysym_to_keycode(XK.string_to_keysym(name));xtest.fake_input(D,X.KeyPress,k);xtest.fake_input(D,X.KeyRelease,k)
 if mod:xtest.fake_input(D,X.KeyRelease,D.keysym_to_keycode(XK.string_to_keysym(mod)))
 D.sync();time.sleep(.25)
def drag(x,y,dx,dy):
 w=window();w.configure(stack_mode=X.Above);D.sync();p=D.screen().root.translate_coords(w,0,0)
 xtest.fake_input(D,X.MotionNotify,x=int(p.x+x),y=int(p.y+y));xtest.fake_input(D,X.ButtonPress,1);D.sync();time.sleep(.12)
 for i in range(1,13):
  xtest.fake_input(D,X.MotionNotify,x=int(p.x+x+dx*i/12),y=int(p.y+y+dy*i/12));D.sync();time.sleep(.04)
 xtest.fake_input(D,X.ButtonRelease,1);D.sync();time.sleep(.3)
def shot(name):
 w=window();g=w.get_geometry();raw=w.get_image(0,0,g.width,g.height,X.ZPixmap,0xffffffff)
 path=Path('/tmp/sd-ux-assignment')/name
 Image.frombytes('RGB',(g.width,g.height),raw.data,'raw','BGRX').save(path)
 print(path)
if __name__=='__main__':
 import sys
 if sys.argv[1]=='shot':shot(sys.argv[2])
 elif sys.argv[1]=='click':click(float(sys.argv[2]),float(sys.argv[3]))
 elif sys.argv[1]=='key':key(*sys.argv[2:])
def paste(text):
 import subprocess
 subprocess.run(['/tmp/sd-ux-assignment/packages/root/usr/bin/xclip','-selection','clipboard'],input=text.encode(),check=True)
 key('v','Control_L')

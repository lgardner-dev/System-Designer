# Control Flow delivery record

Branch: `work/add-logic-diagramming`. Core checkpoint: `8a00167`; guided native
workspace checkpoint: `e35d7d1`. The recovered baseline was `cd6bdbe` and the
workspace was clean. No merge, push, release or dependency change was performed.

The source-review findings R1–R4 were reproduced as four failing regressions
before fixes. They now pass. Native smoke uses the ordinary debug
`target/debug/system-designer`, Xvfb at 1600×1050, xdotool pointer/keyboard input
and ffmpeg X11 capture. These tools were extracted under `/tmp`; no packages were
installed into the system and the application has no automation-only UI.

The screenshots in `docs/evidence/control-flow/` show actual native interactions.
The smoke began from a programmatically written empty v1 project solely to give
the normal Save command an existing filename without an OS file chooser. Start
flow, authoring, information input/output declarations, extraction, new structured
contract refinement, shared-component navigation, stopping criteria, and a second
call occurrence were performed through native controls. A larger form retaining
the previous short modal's viewport was found during smoke and repaired in the
shared dialog shell, with a full-workspace regression.

Final command outputs, complete screenshot inventory, protocol byte counts and
remaining platform qualification are added after the final verification run.

# bevy_wgpu_problem_minimal_repro

Minimally documents a problem I'm having with my custom wgpu renderer that appeared only when moving the repository onto a new machine.

`bevy` is used as glue code here, just like it's used in my renderer. A version of this code can be made that doesn't use bevy, but it would be longer.

I am open to the possibility that I used `wgpu` wrong, and that this code working on my other machines is the strange aspect of this issue. However, I've not been able to find any usage of WGPU that differs from the `wgpu` examples.

## The Problem

After rendering one frame (which does not visually present anything to the screen) the render surface is lost and needs to be recreated. Once it is recreated, another phoney frame is rendered and the process repeats. You can see it in the console here:
![image](https://user-images.githubusercontent.com/7923357/160262309-f69cb049-5faa-4d20-b15e-e4c145a8f392.png)

Here's what the application looks like while this is going on:
![image](https://user-images.githubusercontent.com/7923357/160262361-908ee059-5290-4fc5-84d7-a4bdf9582243.png)

The correct outcome is for the application window to be fully red.

Here's the specs of this computer, which has up to date graphics drivers, in Speccy:
![image](https://user-images.githubusercontent.com/7923357/160262393-68ed5271-45bd-439e-8709-ab21acaf3eb2.png)

## Kinematic or PID?

- Kinematic is more traditional + fully featured character controller
- PID seems to interact with other rigid bodies better + with less lag for continuous collisions (pushing objects around)
- PID has a higher tendency to tunnel (fall/phase through objects)

Conclusion: Use PID if your character will be doing a lof of pushing objects around, otherwise stick to kinematic

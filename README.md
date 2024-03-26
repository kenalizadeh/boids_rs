# Boids simulation with Bevy Engine

# Logs
## Fri 22 Mar
* Calculating local flock's mean direction with [Rosetta Code Article](https://rosettacode.org/wiki/Averages/Mean_angle)
* Currently boid movement is handled by the raycasting alone. So setting the target direction to local flock's mean direction does not work, since raycasting keeps updating it. Maybe add a weighted direction choices and update it from different sources?

## Sun 24 Mar
* Currently stuck. No idea how to manage raycasting result velocity and flock mean velocity to handle boid movement. Sounds easy but *i no smarts*.
* Created a separate project to experiment with movement system. Inspired by [Game Endeavor Video](https://www.youtube.com/watch?v=6BrZryMz-ac). Didn't go far, but had fun.
* Then watched [This Suboptimal Engineer Video](https://www.youtube.com/watch?v=HzR-9tfOJQo) on boids simulation. Tried not to cheat and copy everything so didn't watch a lot. The intro explained how we just have to calculate separate velocities for the rules (separation, alignment, cohesion) and `combine` them together to end up with a final velocity for each boid.
So now going with that. To be continued...

## Tue 26 Mar
* Created a demo project to visualize the separation, alignment and cohesion rules' effects separately and combined together.
* In-progress: Applying the methods implemented in demo to this project.

# Boids simulation with Bevy Engine

# Motivation
* It all started with Sebastian Lague's [video on boids](https://www.youtube.com/watch?v=bqtqltqcQhw)

# Logs
## Fri 22 Mar
* Calculating local flock's mean direction with [Rosetta Code Article](https://rosettacode.org/wiki/Averages/Mean_angle)
* Currently boid movement is handled by the raycasting alone. So setting the target direction to local flock's mean direction does not work, since raycasting keeps updating it. Maybe add a weighted direction choices and update it from different sources?

## Sat 23 Mar
* Currently stuck. No idea how to manage raycasting result velocity and flock mean velocity to handle boid movement. Sounds easy but *i no smarts*.
* Created a separate project to experiment with movement system. Inspired by [Game Endeavor Video](https://www.youtube.com/watch?v=6BrZryMz-ac). Didn't go far, but had fun.

## Sun 24 Mar
* Watched [This Suboptimal Engineer Video](https://www.youtube.com/watch?v=HzR-9tfOJQo) on boids simulation. Tried not to cheat and copy everything so didn't watch a lot. The intro explained how we just have to calculate separate velocities for the rules (separation, alignment, cohesion) and `combine` them together to end up with a final velocity for each boid.
So now going with that. To be continued...

## Tue 26 Mar
* Messed up the project trying to implement the three rules.
* Created a demo project to visualize the separation, alignment and cohesion rules' effects separately and combined together. This was a good idea. All seems to be good.
* Moving to migrating the rules systems implemented in demo project, and removing clear path finding with Raycasting.

## Fri 29 Mar
* Had to refactor almost every aspect of the simulation. But finally got the rules to work and code to make sense.
* Decided to ditch the clear path finding with Raycasting for now, and just teleport the boids to the opposite side of the wall if they move outside the window.
* Boids are finally boiding. There are a lot of room for improvement though, obviously.

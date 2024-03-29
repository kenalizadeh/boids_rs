# Boids simulation with Bevy Engine

# The Journey
It all started when I stumbled upon ==Sebastian Lague=='s [Video on boids](https://www.youtube.com/watch?v=bqtqltqcQhw) after watching a couple indie game content on YouTube.
Then I watched it about 20 more times. I was fascinated by the concept.
And because Sebastian Lague's really entertaining and digestable way of explaining complex concepts I felt confident that I have the skills required to understand and implement boids flocking simulation. But I ended up in a rabbit hole of countless videos on graphics programming, game development, linear algebra alongside wiki pages, articles, forums etc.
As a self-taught iOS Developer, with practically no mathematics (or general CS) background, I had no idea what was ahead.
At first I tried some popular game engines (that also wouldn't set my old macbook on fire)
Tried Godot a bit. But around that time I discovered [Bevy Engine](https://bevyengine.org).
Since I recently started learning Rust, and was really into it, I decided to try Bevy before committing to Godot by default.
And I really liked the way Bevy works, it's Entity Component System was more intuitive for me than Godot's main loop.
And since at the time Bevy didn't have UI/Scene Editor/Visualizer, just writing code in Neovim with Bevy seemed a lot more comfortable both for me and my poor old tired mac.
And it was ==great== fun.

# Logs
## 

## 9 March
Implemented smooth rotation towards the target vec/angle for boids. Previously they just snapped to the target instantly.
Spent more than necessary amount of time on Gimp trying to design a bird sprite. It was horrible but it was mine, so I kept it.

## 17 March
Implemented a grid based spawn system for the boids.

## 18 March
Decided to slap gizmos to practically everything so I can visually debug and understand what's going on.

## 22 March
Calculating local flock's mean direction with [Rosetta Code Article](https://rosettacode.org/wiki/Averages/Mean_angle)
Currently boid movement is handled by the raycasting alone. So setting the target direction to local flock's mean direction does not work, since raycasting keeps updating it. Maybe add a weighted direction choices and update it from different sources?

## 23 March
Currently stuck. No idea how to manage raycasting result velocity and flock mean velocity to handle boid movement. Sounds easy but *i no smarts*.
Created a separate project to experiment with movement system. Inspired by [Game Endeavor Video](https://www.youtube.com/watch?v=6BrZryMz-ac). Didn't go far, but had fun.

## 24 March
Until this point I tried to figure the rule implementations myself, hence the whole Raycasting, local flock's mean angle calculation. Which was lots of fun, since I had no deadlines. It meant 
 Then I watched [This Suboptimal Engineer Video](https://www.youtube.com/watch?v=HzR-9tfOJQo) on boids simulation. Tried not to cheat and copy everything so didn't watch a lot. The intro explained how we just have to calculate separate velocities for the rules (separation, alignment, cohesion) and `combine` them together to end up with a final velocity for each boid.
So now going with that. To be continued...

## Tue 26 March
After realizing I'm out of my depth, I watched the [previously mentioned video](https://www.youtube.com/watch?v=HzR-9tfOJQo) again, and decided to study the given code examples and try them out.
Turns out I was doing almost everything wrong. Which I didn't mind actually, since I also learned a lot that way. 
Messed up the project trying to implement the three rules.
 Create a demo project to visualize the separation, alignment and cohesion rules' effects separately and combined together (Inspired by previously mentioned [Game Endeavor Video](https://www.youtube.com/watch?v=6BrZryMz-ac)). This was a good idea. All seems to be working.
Moving to migrating the rules systems implemented in demo project, and removing clear path finding with Raycasting.

## Fri 29 March
Had to refactor almost every aspect of the simulation. But finally got the rules to work and code to make sense.
Decided to ditch the clear path finding with Raycasting for now, and just teleport the boids to the opposite side of the wall if they move outside the window.
Boids are finally boiding. There are a lot of room for improvement though, obviously.

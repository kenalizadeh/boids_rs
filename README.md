# Boids Simulation
### 2D [Boids](https://en.wikipedia.org/wiki/Boids) Flocking Simulation with [Bevy Engine](https://bevyengine.org)

</br>

## The Journey
It all started when I stumbled upon Sebastian Lague's [Video on boids](https://www.youtube.com/watch?v=bqtqltqcQhw) after watching a couple indie game content on YouTube.
His entertaining and digestable way of explaining complex concepts inspired me to implement the simulation myself.
At first I tried some popular game engines like Godot, Game Maker Studio, Solar2D. 
But around that time I also came across [Bevy Engine](https://bevyengine.org), and decided to give Bevy a try.
I really liked the way Bevy works, it's Entity Component System was more intuitive for me than traditional main loop, and I felt more comfortable with Rust than the other options (GDScript, Lua, C#, C or C++).
So I just created a new project and started working.

### And it was **great** fun.

</br>

## Demo

![boids](https://github.com/kenalizadeh/boids_rs/assets/4370392/d4ab255b-4e0f-4d61-8dae-8a07c5ca6fc2)

</br>

# Journey Logs
### 8 March
Learned about Raycasting. And decided to implement collision avoidance with it.

### 9 March
Implemented smooth rotation towards the target vec/angle for boids. Previously they just snapped to the target instantly.
Spent more than necessary amount of time on Gimp trying to design a bird sprite. It was horrible but it was mine, so I kept it.

### 17 March
Implemented a grid based spawn system for the boids.

### 18 March
Decided to slap gizmos to practically everything so I can visually debug and understand what's going on.

### 22 March
Calculating local flock's mean direction with [Rosetta Code Article](https://rosettacode.org/wiki/Averages/Mean_angle)
Currently boid movement is handled by the raycasting alone. So setting the target direction to local flock's mean direction does not work, since raycasting keeps updating it. Maybe add a weighted direction choices and update it from different sources?

### 23 March
Currently stuck. No idea how to manage raycasting result velocity and flock mean velocity to handle boid movement. Sounds easy but *i no smarts*.
Created a separate project to experiment with movement system. Inspired by [Game Endeavor Video](https://www.youtube.com/watch?v=6BrZryMz-ac). Didn't go far, but had fun.

### 24 March
Until this point I tried to figure the rule implementations myself, hence the whole Raycasting, local flock's mean angle calculation. Which was lots of fun, since I had no deadlines. It meant 
Then I watched [This Suboptimal Engineer Video](https://www.youtube.com/watch?v=HzR-9tfOJQo) on boids simulation. Tried not to cheat and copy everything so didn't watch a lot. The intro explained how we just have to calculate separate velocities for the rules (separation, alignment, cohesion) and `combine` them together to end up with a final velocity for each boid.
So now going with that. To be continued...

### Tue 26 March
After realizing I'm out of my depth, I watched the [previously mentioned video](https://www.youtube.com/watch?v=HzR-9tfOJQo) again, and decided to study the given code examples and try them out.
Turns out I was doing almost everything wrong. Which I didn't mind actually, since I also learned a lot that way. 
Messed up the project trying to implement the three rules.
Created a demo project to visualize the separation, alignment and cohesion rules' effects separately and combined together (Inspired by previously mentioned [Game Endeavor Video](https://www.youtube.com/watch?v=6BrZryMz-ac)). This was a good idea. All seems to be working.
Moving to migrating the rules systems implemented in demo project, and removing clear path finding with Raycasting.

### Fri 29 March
Had to refactor almost every aspect of the simulation. But finally got the rules to work and code to make sense.
Also had to sacrifice my precious 2D triangular bird sprite in favor of equilateral triangles with different pastel colors.
Decided to ditch the clear path finding with Raycasting for now, and just teleport the boids to the opposite side of the wall if they move outside the window.
Boids are finally boiding. There are a lot of room for improvement though, obviously.

</br>

# Journey Summary
The original Boids by Craig Reynolds is a simple but fascinating system.
I learned interesting maths, specifically linear algebra concepts for vector operations, game development philosophy and methods.
Bevy Engine might be in it's early stages.
This was my first ever proper experience with a game engine and I really enjoyed Bevy the entire duration of using it.
It's Entity Component System is really intuitive to work with.
And being able to `gd` into engine code was really helpful. And to my knowledge it's something most game engines cannot offer.
I can confidently use Bevy again in future projects.
In conclusion, this was a refreshing, pleasant and enlightening experience.

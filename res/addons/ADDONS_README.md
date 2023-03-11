# Addons

You can specify additional resources (scripts, sounds, etc) to load by using addons.
Addons are just folders with the same directory structure as `/res/`, minus the `addons` folder.

A directory structure for an addon named `example` might look like this:

```
📁 /res/addons
 └📁 example
   └📁 scripts
     ├📁 agents
     │ └📄 operator.lua
     ├📁 api
     │ └📄 utilities.lua
     ├📁 sounds
     │ └🔊 bigscream.wav
     └📁 soundbanks
```
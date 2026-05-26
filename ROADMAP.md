# Roadmap

## Playable game
- [ ] Placing cards
- [ ] Turn orders

## Networking
- [ ] API
  - [ ] Plugin communication
  - [ ] Random number sharing...
  - [ ] Project restructure
- [ ] Game builder
  - [ ] Deck builder
  - [ ] Lobby
- [ ] Local prevalidations with secrets

## Replays
- [ ] RNG recoding
- [ ] Action recording

## Plugins
- [ ] Bonus cards
- [ ] Plugin loading[*](#The-end-of-the-road)
- [ ] Plugin menu
- [ ] Action implementations

## Spice
- [x] Proper workspace layout
- [ ] Dont make it look ugly
  - [ ] Redraw numbers
- [ ] Ease of info
  - [ ] Dictionary system
  - [ ] Card info system
  - [ ] Colo(u)r blindness options
  - [ ] Hand filtering

# The end of the road
Since there is no ABI for both rust and bevy, the plugin system cannot be extrapolated. Unless... a new engine is made so that we can do this. That will never happen in a normal amount of time so we can only hope.

The program will be designed with plugins in mind despite this fact.

## Last exit
Perhaps there is some way to provide our own interface with the engine anyway and have that be ABI stable. This is likely a problem for later though


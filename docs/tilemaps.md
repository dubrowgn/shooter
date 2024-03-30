# Core Design
* square tiles
* scrolling (i.e. player view v whole map)
* layers
* translation from tilemap tools to our format?

# Tilemap data to track
* tile size
* image atlas
* dimmensions (tile x tile)
* visual grid (index of tile in image atlas at this position)
* logic grid (is collidable, path finding, etc.)

# Tile data to track

# Open questions
* layers:
** is each layer a different tilemap? probably no
** does a tilemap have an array of visual grids to track the layers
** is each "tile" in the visual grid an array holding the tile for that layer with some default for no tile
** how would this work with the character if the character is not the top layer
** or do you have exclusively 3 "layers": background (background tilemap), game objects (ECS), character

# Alternaties
## dimmensions
Could also do pixel x pixel but defining it tile x tile seems to scale nicer with different resolutions. Tiles can be any number of pixels for higher res systems and logic would still hold. Makes it relative rather than explicit.
## tile shape
Rectangle adds complexity for nothing immediately gained that we want/need. Same for hexagonal.


# References
* https://developer.mozilla.org/en-US/docs/Games/Techniques/Tilemaps
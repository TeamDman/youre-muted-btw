# create .ico
magick convert -background transparent "icon.png" -define icon:auto-resize=16,24,32,48,64,72,96,128,256 "favicon.ico"

# create resized image
magick convert -background transparent "icon.png" -resize 64x64 ".\assets\textures\icon.png"
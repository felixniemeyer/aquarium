import os
import time
from PIL import Image

img_path = "./img.jpg" 

os.system("echo triggering shot")
# os.system("gphoto2 --camera=\"Canon EOS 600D\" --capture-image-and-download --no-keep --filename=" + img_path) 

mask = Image.open('fish-templates/v1-mask.png').getchannel('R')
background = Image.new('RGBA', (5760, 3240), (155,155,100,0))

while not os.path.exists(img_path):
    time.sleep(0.1)

img = Image.open(img_path)

# todo: the mask and the cropping will depend on the used stencil 
fish = Image.composite(img, background, mask)
fish = fish.crop((1337, 768, 4096, 768 + (4096 - 1337)))
fish = fish.resize((1024, 1024),Image.ANTIALIAS)

fish.save("result.png", "PNG") 

#delete image

import os
import time
import numpy as np
from PIL import Image

img_path = "./IMG_1539.JPG" #TODO: img.jpg
bg_color = (0,0,0)

os.system("echo triggering shot")
#TODO: os.system("gphoto2 --camera=\"Canon EOS 600D\" --capture-image-and-download --no-keep --filename=" + img_path) 


while not os.path.exists(img_path):
    time.sleep(0.1)

# mask = Image.new('L', (5184, 3456), 1)

img = Image.open(img_path)
img_data = np.array(img)
img_dims = img_data.shape
mask_data = np.ndarray(shape=(img_dims[0], img_dims[1]), dtype=float)

s = 0
for x in range(img_dims[0]):
    for y in range(img_dims[1]): 
        s = 0
        for i in range(3):
           s += (img_data[x,y,i] - bg_color[i]) ** 2
        mask_data[x,y] = s

mask = Image.fromarray(mask_data)

mask.view()
            
        
# todo: the mask and the cropping will depend on the used stencil 
# fish = Image.composite(img, background, mask)
# fish = fish.crop((1337, 768, 4096, 768 + (4096 - 1337)))
# fish = fish.resize((1024, 1024),Image.ANTIALIAS)
# fish.save("result.png", "PNG") 

#delete image

#skimage stuff: 
# img = skimage.io.imread(fname=img_path)
# mask = img - bg_color #np.linalg.norm(img - bg_color) < 20
# viewer = skimage.viewer.ImageViewer(mask)
# viewer.show()

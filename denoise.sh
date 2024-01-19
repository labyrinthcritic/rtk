#!/bin/sh

image_path=$1

magick convert "$image_path" -endian LSB PFM:image.pfm
oidnDenoise -hdr image.pfm -o denoised.pfm
magick convert denoised.pfm "PNG:denoised.png"
rm image.pfm
rm denoised.pfm

#!/bin/sh

set -e

image_path=$1

if [ -z $image_path ]
then
  echo 'usage: ./denoise.sh <path>'
  exit 1
fi

magick convert "$image_path" -endian LSB PFM:image.pfm
oidnDenoise --srgb --quality high -ldr image.pfm -o denoised.pfm
magick convert denoised.pfm "PNG:denoised.png"
rm image.pfm
rm denoised.pfm

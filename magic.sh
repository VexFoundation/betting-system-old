#!/bin/bash

npm install
chmod +x deploy.sh
chmod +x build.sh
./build.sh
./deploy.sh


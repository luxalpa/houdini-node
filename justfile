#!/usr/bin/env just --justfile

set shell := ["powershell.exe", "-c"]

hda:
    hotl -l houdini/unpacked houdini/node.hda

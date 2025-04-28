#!/usr/bin/env bash

sops -e ./dev/secrets.yaml > ./dev/secrets.enc.yaml

#!/usr/bin/env bash

RUST_LOG=DEBUG /home/dacal/dsrv -b '0.0.0.0:80' 2>&1 | multilog t n2 s10485760 /var/log/dacal

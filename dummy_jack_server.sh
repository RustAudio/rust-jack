#!/bin/sh
# Start the dummy JACK server

jackd -r -ddummy -r44100 -p1024

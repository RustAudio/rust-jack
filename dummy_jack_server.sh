#!/bin/sh
# Start the dummy JACK server

exec jackd -r -ddummy -r44100 -p1024

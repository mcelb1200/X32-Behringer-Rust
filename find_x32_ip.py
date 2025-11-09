#!/usr/bin/env python3
import socket
import sys

def find_x32():
    """
    Sends a UDP broadcast packet to find an X32 mixer on the network
    and prints its IP address if found.
    """
    # OSC /info packet: /info\0\0\0,\0\0\0
    packet = b'\x2f\x69\x6e\x66\x6f\x00\x00\x00\x2c\x00\x00\x00'

    # Create a UDP socket
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
    sock.settimeout(2.0) # 2-second timeout

    try:
        # Send the packet to the broadcast address on port 10023
        sock.sendto(packet, ('255.255.255.255', 10023))

        # Wait for a response
        data, addr = sock.recvfrom(1024)

        # If we get a response, print the IP address
        if data:
            print(addr[0])
            sys.exit(0)

    except socket.timeout:
        # No response received, exit silently
        sys.exit(1)
    except Exception as e:
        # Print other errors to stderr
        print(f"An error occurred: {e}", file=sys.stderr)
        sys.exit(1)
    finally:
        sock.close()

if __name__ == '__main__':
    find_x32()

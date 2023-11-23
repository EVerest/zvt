#!/usr/bin/env python3

from pathlib import Path
import argparse
import binascii
import os
import pyshark

def parse_args():
    p = argparse.ArgumentParser(description=
        "Extract ZVT packages from the given file."
    )

    p.add_argument("-i", "--input", type=Path, required=True, help = "input file")
    p.add_argument("-o", "--output_dir", type=Path, default="zvt_packets", help = "Directory for dumped files")

    return p.parse_args()

def main():
    args = parse_args()

    if not os.path.exists(args.output_dir):
        os.makedirs(args.output_dir)

    cap = pyshark.FileCapture(args.input, display_filter='tcp.port==22000')
    for pkt in cap:
        if not hasattr(pkt.tcp, 'payload'):
            continue  # Skip packets without a payload

        timestamp = pkt.sniff_timestamp
        src = pkt.ip.src
        dst = pkt.ip.dst
        payload = pkt.tcp.payload

        binary_payload = binascii.unhexlify(payload.replace(':', ''))

        output_filename = f"{args.output_dir}/{timestamp}_{src}_{dst}.blob"
        with open(output_filename, 'wb') as f:
            f.write(binary_payload)

if __name__ == '__main__':
    main()

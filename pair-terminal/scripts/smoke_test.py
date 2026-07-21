#!/usr/bin/env python3
"""End-to-end smoke test for pair-terminal.

This script does NOT exercise the real `pair` binary (it needs a TTY);
instead it speaks the WebSocket protocol directly to verify the relay
server and the E2E key exchange path work together.

Steps:
  1. Connect as host, send Handshake, expect HandshakeOk
  2. Connect as guest, send Handshake, expect HandshakeOk + NewPeerConnected
     on the host side
  3. Host calls rotate() (modeled in Python: pick a fresh key, encrypt it
     with the bootstrap, send AesKeys). Guest extracts the keys, both
     peers exchange an encrypted message and verify the round-trip
     matches the test vector.

Run with the relay server already listening:
  pair-server --host 127.0.0.1 --port 18877
"""
import asyncio
import base64
import json
import sys

import websockets
from cryptography.hazmat.primitives.ciphers.aead import AESGCM

WS_URL = "ws://127.0.0.1:18877/ws"
TERMINAL_ID = "smoke-terminal-1"


def b64e(b: bytes) -> str:
    return base64.urlsafe_b64encode(b).rstrip(b"=").decode()


def b64d(s: str) -> bytes:
    pad = "=" * ((4 - len(s) % 4) % 4)
    return base64.urlsafe_b64decode(s + pad)


async def main() -> int:
    bootstrap = b"smoke-bootstrap-key"  # 16 bytes (would be random in real use)
    bootstrap_b64 = b64e(bootstrap)

    # --- Host connects ---
    host = await websockets.connect(WS_URL)
    await host.send(
        json.dumps(
            {
                "type": "Handshake",
                "payload": {
                    "user_id": "host-user",
                    "role": "Host",
                    "cols": 80,
                    "rows": 24,
                    "terminal_id": TERMINAL_ID,
                    "mode": "Collaborative",
                    "allow_guest_control": True,
                },
            }
        )
    )
    host_hello = json.loads(await host.recv())
    assert host_hello["type"] == "HandshakeOk", host_hello
    print(f"[host] HandshakeOk: {host_hello['payload']}")

    # --- Guest connects ---
    guest = await websockets.connect(WS_URL)
    await guest.send(
        json.dumps(
            {
                "type": "Handshake",
                "payload": {
                    "user_id": "guest-user",
                    "role": "Guest",
                    "cols": 80,
                    "rows": 24,
                    "terminal_id": TERMINAL_ID,
                    "mode": "Collaborative",
                    "allow_guest_control": True,
                },
            }
        )
    )
    guest_hello = json.loads(await guest.recv())
    assert guest_hello["type"] == "HandshakeOk", guest_hello
    print(f"[guest] HandshakeOk: {guest_hello['payload']}")

    # --- Host should be told the guest is here ---
    host_notice = json.loads(await host.recv())
    assert host_notice["type"] == "NewPeerConnected", host_notice
    print(f"[host] saw NewPeerConnected")

    # --- Simulate key rotation ---
    new_output_key = b"new-output-key-12"  # 16 bytes
    new_input_key = b"new-input-key-123"  # 16 bytes
    # Encrypt the new keys with the bootstrap (AES-GCM, counter 0 → nonce 0...)
    bootstrap_aes = AESGCM(bootstrap)
    iv_zero = b"\x00" * 12
    enc_output = iv_zero + bootstrap_aes.encrypt(iv_zero, new_output_key, None)
    enc_input = iv_zero + bootstrap_aes.encrypt(iv_zero, new_input_key, None)
    await host.send(
        json.dumps(
            {
                "type": "AesKeys",
                "payload": {
                    "b64_output_key": b64e(enc_output),
                    "b64_input_key": b64e(enc_input),
                    "iv_count": 0,
                    "max_iv_count": 1048576,
                },
            }
        )
    )
    print(f"[host] sent AesKeys")

    # --- Guest should receive AesKeys ---
    guest_keys = json.loads(await guest.recv())
    assert guest_keys["type"] == "AesKeys", guest_keys
    enc_out = b64d(guest_keys["payload"]["b64_output_key"])
    enc_in = b64d(guest_keys["payload"]["b64_input_key"])
    iv_count = guest_keys["payload"]["iv_count"]
    guest_output_key = AESGCM(bootstrap).decrypt(enc_out[:12], enc_out[12:], None)
    guest_input_key = AESGCM(bootstrap).decrypt(enc_in[:12], enc_in[12:], None)
    assert guest_output_key == new_output_key
    assert guest_input_key == new_input_key
    print(f"[guest] extracted keys match")

    # --- Host → guest (host encrypts with new_output_key, guest decrypts) ---
    # In a real session the host has now also adopted new_output_key as its
    # own key (the bug fix from this commit). We mirror that here.
    host_msg = b"ls -la\\nfile.txt\\n"
    host_aes = AESGCM(new_output_key)
    host_nonce = iv_count.to_bytes(8, "little").rjust(12, b"\x00")
    host_ct = host_nonce + host_aes.encrypt(host_nonce, host_msg, None)
    await host.send(
        json.dumps(
            {
                "type": "PtyOutput",
                "payload": {"data": b64e(host_ct), "encrypted": True},
            }
        )
    )
    print(f"[host] sent encrypted PtyOutput")

    guest_recv = json.loads(await guest.recv())
    assert guest_recv["type"] == "PtyOutput", guest_recv
    wire = b64d(guest_recv["payload"]["data"])
    guest_aes = AESGCM(guest_output_key)
    plain = guest_aes.decrypt(wire[:12], wire[12:], None)
    assert plain == host_msg, (plain, host_msg)
    print(f"[guest] decrypted PtyOutput matches: {plain!r}")

    # --- Guest → host (guest encrypts with new_input_key, host decrypts) ---
    guest_msg = b"cat file.txt\\n"
    guest_enc = AESGCM(guest_input_key)
    guest_nonce = iv_count.to_bytes(8, "little").rjust(12, b"\x00")
    guest_ct = guest_nonce + guest_enc.encrypt(guest_nonce, guest_msg, None)
    await guest.send(
        json.dumps(
            {
                "type": "KeyInput",
                "payload": {"data": b64e(guest_ct), "encrypted": True},
            }
        )
    )
    print(f"[guest] sent encrypted KeyInput")

    host_recv = json.loads(await host.recv())
    assert host_recv["type"] == "KeyInput", host_recv
    wire = b64d(host_recv["payload"]["data"])
    host_in_aes = AESGCM(new_input_key)
    plain = host_in_aes.decrypt(wire[:12], wire[12:], None)
    assert plain == guest_msg, (plain, guest_msg)
    print(f"[host] decrypted KeyInput matches: {plain!r}")

    await host.close()
    await guest.close()
    print("\\nSMOKE TEST PASSED")
    return 0


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))

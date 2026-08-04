#!/bin/sh
set -e

echo "========================================"
echo "    Uninstalling DropWire Suite         "
echo "========================================"

echo "Removing DropWire CLI (/usr/local/bin/dropwire)..."
if [ -f "/usr/local/bin/dropwire" ]; then
    sudo rm -f /usr/local/bin/dropwire
    echo "✓ CLI removed."
else
    echo "CLI not found."
fi

echo "Removing DropWire TUI (/usr/local/bin/dropwirex)..."
if [ -f "/usr/local/bin/dropwirex" ]; then
    sudo rm -f /usr/local/bin/dropwirex
    echo "✓ TUI removed."
else
    echo "TUI not found."
fi

echo "========================================"
echo "DropWire Suite has been fully uninstalled."
echo "========================================"

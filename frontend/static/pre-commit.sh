#!/bin/bash

# Path to the file containing secrets
TARGET_FILE="./settings/base.yaml"

if [ -f "$TARGET_FILE" ]; then
    echo "Scrubbing secrets from $TARGET_FILE before commit..."

    # Replace the MongoDB URI (the first URI line) with a placeholder
    # This targets the specific pattern used in build.rs [1]
    sed -i 's|mongodb://admin:.*|mongodb://admin:<password>"|' "$TARGET_FILE"
    
    # Replace the Redis URI (the second URI line) with a placeholder
    # This targets the IP 10.20.20.32 mentioned in build.rs [1]
    sed -i 's|redis://:.*|redis://:<password>|' "$TARGET_FILE"

    # Stage the scrubbed version of the file so the clean version is committed
    git add "$TARGET_FILE"
fi

exit 0

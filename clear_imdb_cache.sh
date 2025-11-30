#!/bin/bash
# Script to clear IMDb ratings from cache

DB_FILE=".data/torrentflix.db"

if [ ! -f "$DB_FILE" ]; then
    echo "Database file not found at: $DB_FILE"
    echo "Looking for database file..."
    
    # Try alternative locations
    if [ -f "src-tauri/torrentflix.db" ]; then
        DB_FILE="src-tauri/torrentflix.db"
        echo "Found database at: $DB_FILE"
    else
        echo "Could not find database file. Please check the location."
        exit 1
    fi
fi

echo "Clearing IMDb ratings from cache..."
sqlite3 "$DB_FILE" "UPDATE rating_cache SET rating_imdb = NULL, updated_at = $(date +%s) WHERE rating_imdb IS NOT NULL;"

COUNT=$(sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM rating_cache WHERE rating_imdb IS NOT NULL;")
echo "✅ IMDb cache cleared! Remaining IMDb ratings: $COUNT"


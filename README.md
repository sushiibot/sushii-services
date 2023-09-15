# sushii-sharder

Sharder service that connects to the Discord gateway and sends events to Redis.

## Flow

1. sushii-sharder sends events to Redis "events" list
2. sushii-processor consumes events and handles commands & event handlers
   * Generic command parser to share with multiple different bots

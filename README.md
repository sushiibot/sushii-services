# sushii-services

**Disclaimer:** This is just some some random experimentation working with
twilight and a Redis message queue.

## Content

* **sharder:** Connects to the Discord gateway and sends events to Redis.
* **commands:** Command mini-framework for parser and event handler
* **tasks:** Background worker that doesn't rely on gateway events, only
  dispatches messages via Discord http API

## Flow

1. sushii-sharder sends events to Redis "events" list
2. sushii-processor consumes events and handles commands & event handlers
   * Twilight wrapper to add features to share with multiple different bots
     * Command parser (built in help)
     * Default common useful commands (about, prefix, etc)

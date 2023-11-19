# Ponchik Donut App

A blazing fast slack donut app clone written in Rust.

## Functionality

When ran, Ponchik will generate a list of pairs of members of a slack channel. It will make those introductions in Slack DMs, and record the pairs on a Google Sheet. 

## Lifecycle

- Monday send out pairings
- Friday midpoint checkin
- Next Friday result survey

Bot checks a google sheet cell to check time, chooses one of 3 paths as a result. 

## Todos
- [x] Get group making with real names running
- [x] Google Sheets integration
- [ ] Refactor: fix all warnings and shortcut approaches to borrow checker
- [ ] Refactor: Restructure trait, functions, and module format. Add env vars
    - [x] Integrate user profile structs & find better solution for storing that info
- [ ] DMs user input response handling
    - [ ] Write slack blocks for it
    - [ ] Figure out how to set up server to handle responses
    - [ ] update survey message after receing a message
- [ ] Design app lifecyle & sending updates
- [ ] Figure out app hosting solution
- [ ] Update set making to prevent duplicates (bloom filter / google query)
- [ ] calculate which cells to edit to make it possible to keep track of additional information regaridng pairings 
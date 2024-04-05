## Local Ctrl Readme

Seperate out the Led switch folder from the wrokspace

Remember to use **opt-level = z** for esp chips in cargo.toml

> Note: to Add the package.metadata for esp mdns

Step to run the dev version
- First comment the local ctrl init function in main.rs for provisioning (http server issue)
- Again flash the esp with uncommented local ctrl init 
- Now Connect your phone app and esp on same network and wait for REACHABLE ON WLAN to use Local ctrl
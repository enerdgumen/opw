## Start the daemon with systemd

```
cp ./systemd/opw.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable opw
systemctl --user start opw
```

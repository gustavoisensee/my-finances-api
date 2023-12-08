## My-finances-api-v3 ##

Restful api for my-finances-web

### Keep render alive
```bash
# 60 * 14 = 840 (60s, 14min, 840s)
# render.com spin it down after 15min of inactivity
while true; do curl "https://my-finances-api-v4.onrender.com/"; sleep 840; done;
```
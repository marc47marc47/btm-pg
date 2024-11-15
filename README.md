


add setenv.set
```sh
$ cat setenv.set
 export PGPASSWORD=YOUR_PASSWORD_HERE
 export DATABASE_URL=postgres://postgres:$PGPASSWORD$@192.168.131.71:5432/cobra?sslmode=disable

$ . ./setenv.set
$ cargo run
```

![alt text](btm-pg.png "btm-pg image")

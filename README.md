A tile game *very* much inspired by [Azul](https://en.wikipedia.org/wiki/Azul_(board_game)). To play, build the Docker image and run it:

```
docker build -t amarillo .
docker run -i -t --init amarillo /usr/src/app/run.sh
```

The game is implemented for 3 players and your opponents choose their moves using a Monte Carlo tree search.

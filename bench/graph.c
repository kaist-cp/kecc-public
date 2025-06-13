int graph_weight[1000][1000];
int graph_dijkstra_dist[1000];
int graph_dijkstra_visited[1000];

void graph_weight_init(int n, int nonce, int* x, int (*weight)[1000]) {
    for (int i = 0; i < n; ++i) {
        weight[i][i] = 0;

        for (int j = 1; j < n; ++j) {
            weight[i][(i + j) % n] = ++*x;

            if (*x % (nonce + 1)) {
                ++*x;
            }
        }
    }
}

int graph_dijkstra(int n, int nonce) {
    if (!(n <= 1000)) {
        return nonce;
    }

    int x = 0;
    graph_weight_init(n, nonce, &x, graph_weight);

    for (int i = 0; i < n; ++i) {
        graph_dijkstra_dist[i] = -1;
        graph_dijkstra_visited[i] = 0;
    }
    graph_dijkstra_dist[0] = 0;

    for (int step = 0; step < n; ++step) {
        int v = -1;
        for (int i = 0; i < n; ++i) {
            if (!(graph_dijkstra_dist[i] != -1 && !graph_dijkstra_visited[i])) {
                continue;
            }

            if (v != -1 && graph_dijkstra_dist[v] < graph_dijkstra_dist[i]) {
                continue;
            }

            v = i;
        }

        if (v == -1) {
            break;
        }

        int dist = graph_dijkstra_dist[v];
        graph_dijkstra_visited[v] = 1;

        for (int i = 0; i < n; ++i) {
            if (graph_dijkstra_visited[i])
                continue;
            if (graph_dijkstra_dist[i] != -1 && graph_dijkstra_dist[i] < dist + graph_weight[v][i])
                continue;
            graph_dijkstra_dist[i] = dist + graph_weight[v][i];
        }
    }

    int result = 0;
    for (int i = 0; i < n; ++i) {
        result ^= graph_dijkstra_dist[i];
    }

    return result;
}

int graph_floyd_warshall(int n, int nonce) {
    if (!(n <= 1000)) {
        return nonce;
    }

    int x = 0;
    graph_weight_init(n, nonce, &x, graph_weight);

    for (int k = 0; k < n; ++k) {
        for (int i = 0; i < n; ++i) {
            if (graph_weight[i][k] == -1)
                continue;

            for (int j = 0; j < n; ++j) {
                if (graph_weight[k][j] == -1)
                    continue;
                int weight = graph_weight[i][k] + graph_weight[k][j];
                if (graph_weight[i][j] != -1 && graph_weight[i][j] < weight)
                    continue;
                graph_weight[i][j] = weight;
            }
        }
    }

    int result = 0;
    for (int i = 0; i < n; ++i) {
        for (int j = 0; j < n; ++j) {
            result ^= graph_weight[i][j];
        }
    }
    return result;
}

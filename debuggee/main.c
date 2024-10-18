// Multi-threaded sample target for debugging
// gcc -o debuggee_binary debuggee/main.c -lpthread -fno-pie -no-pie

#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

int shared_func(int x) { return ((x ^ 8) + 9) * 3; }

void *thread_func(void *arg) {
  int i;
  while (1) {
    for (i = 0; i < 10; i++) {
      printf("Thread %d: %d\n", (int)arg, shared_func(i));
      usleep(1000000);
    }
  }
}

int main() {
  pthread_t thread1, thread2;
  int i;

  printf("Address of main: %p\n", main);
  printf("Address of thread_func: %p\n", thread_func);
  printf("Address of shared_func: %p\n", shared_func);

  pthread_create(&thread1, NULL, (void *)thread_func, (void *)1);
  pthread_create(&thread2, NULL, (void *)thread_func, (void *)2);

  while (1) {
    for (i = 0; i < 10; i++) {
      printf("Main: %d\n", shared_func(i));
      usleep(1000000);
    }
  }

  pthread_join(thread1, NULL);
  pthread_join(thread2, NULL);

  return 0;
}
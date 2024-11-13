
#include <cstdio>
#include <iostream>
#include <thread>
#include <curl/curl.h>

static volatile int counter = 0;

void increment(int thread_id) {
    counter++;
    std::cout << "Thread " << thread_id << " incremented counter to " << counter << std::endl;
    std::this_thread::sleep_for(std::chrono::seconds(1));
}

size_t write_callback(void *ptr, size_t size, size_t nmemb, void *userdata) {
    std::cout << "got " << size * nmemb << " bytes" << std::endl;
    return size * nmemb;
}

void make_get_request(int thread_id) {
    CURL *curl;
    CURLcode res;
    curl = curl_easy_init();
    curl_easy_setopt(curl, CURLOPT_URL, "http://www.google.com");
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_callback);
    res = curl_easy_perform(curl);
    curl_easy_cleanup(curl);
}

int main(int argc, char **argv) {
    for (int i = 0; i < argc; i++) {
        std::cout << "Arg " << i << ": " << argv[i] << std::endl;
    }

    while (true) {
        std::thread t1(increment, 1);
        std::thread t2(increment, 2);
        std::thread t3(make_get_request, 3);
        t1.join();
        t2.join();
        t3.join();
        std::cout << "Counter: " << counter << std::endl;
    }

    return 0;
}

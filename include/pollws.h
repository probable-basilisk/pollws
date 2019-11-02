#include <stddef.h>

typedef enum pollws_status_type
{
  CLOSED,
  OPENING,
  OPEN,
  ERROR,
} pollws_status_t;

typedef struct PollWSSocket PollWSSocket;

PollWSSocket* pollws_open(const char* url);
void pollws_close(PollWSSocket* ctx);
pollws_status_t pollws_status(PollWSSocket* ctx);
void pollws_send(PollWSSocket* ctx, const char* msg);
int pollws_poll(PollWSSocket* ctx);
unsigned int pollws_get(PollWSSocket* ctx, char* dest, unsigned int dest_size);
unsigned int pollws_pop(PollWSSocket* ctx, char* dest, unsigned int dest_size);
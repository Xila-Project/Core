#include "../../../abi/include/xila.h"
#include "../include/internal.h"
#include "platform_internal.h"

/***************************************************
 *                                                 *
 *                Extension interface              *
 *                                                 *
 ***************************************************/

/****************************************************
 *                     Section 1                    *
 *                Multi thread support              *
 ****************************************************/

/**
 * NOTES:
 * 1. If you are building VM core only, it must be implemented to
 *    enable multi-thread support, otherwise no need to implement it
 * 2. To build the app-mgr and app-framework, you must implement it
 */

/**
 * Creates a thread
 *
 * @param p_tid  [OUTPUT] the pointer of tid
 * @param start  main routine of the thread
 * @param arg  argument passed to main routine
 * @param stack_size  bytes of stack size
 *
 * @return 0 if success.
 */
int os_thread_create(korp_tid *p_tid, thread_start_routine_t start, void *arg,
                     unsigned int stack_size)
{
    return xila_thread_create(start, arg, stack_size, p_tid);
}

/**
 * Creates a thread with priority
 *
 * @param p_tid  [OUTPUT] the pointer of tid
 * @param start  main routine of the thread
 * @param arg  argument passed to main routine
 * @param stack_size  bytes of stack size
 * @param prio the priority
 *
 * @return 0 if success.
 */
int os_thread_create_with_prio(korp_tid *p_tid, thread_start_routine_t start,
                               void *arg, unsigned int stack_size, int prio)
{
    return xila_thread_create(start, arg, stack_size, p_tid);
}

/**
 * Waits for the thread specified by thread to terminate
 *
 * @param thread the thread to wait
 * @param retval if not NULL, output the exit status of the terminated thread
 *
 * @return return 0 if success
 */
int os_thread_join(korp_tid thread, void **retval)
{
    return xila_thread_join(thread);
}

/**
 * Detach the thread specified by thread
 *
 * @param thread the thread to detach
 *
 * @return return 0 if success
 */
int os_thread_detach(korp_tid Thread)
{
    return xila_thread_detach(Thread);
}

/**
 * Exit current thread
 *
 * @param retval the return value of the current thread
 */
void os_thread_exit(void *retval)
{
    xila_thread_exit();
}

/* Try to define os_atomic_thread_fence if it isn't defined in
   platform's platform_internal.h */
#ifndef os_atomic_thread_fence

#if !defined(__GNUC_PREREQ) && (defined(__GNUC__) || defined(__GNUG__)) && !defined(__clang__) && defined(__GNUC_MINOR__)
#define __GNUC_PREREQ(maj, min) \
    ((__GNUC__ << 16) + __GNUC_MINOR__ >= ((maj) << 16) + (min))
#endif

/* Clang's __GNUC_PREREQ macro has a different meaning than GCC one,
   so we have to handle this case specially */
#if defined(__clang__)
/* Clang provides stdatomic.h since 3.6.0
   See https://releases.llvm.org/3.6.0/tools/clang/docs/ReleaseNotes.html */
#if __clang_major__ > 3 || (__clang_major__ == 3 && __clang_minor__ >= 6)
#define BH_HAS_STD_ATOMIC
#endif
#elif defined(__GNUC_PREREQ)
/* Even though older versions of GCC support C11, atomics were
   not implemented until 4.9. See
   https://gcc.gnu.org/bugzilla/show_bug.cgi?id=58016 */
#if __GNUC_PREREQ(4, 9)
#define BH_HAS_STD_ATOMIC
#elif __GNUC_PREREQ(4, 7)
#define os_memory_order_acquire __ATOMIC_ACQUIRE
#define os_memory_order_release __ATOMIC_RELEASE
#define os_memory_order_seq_cst __ATOMIC_SEQ_CST
#define os_atomic_thread_fence __atomic_thread_fence
#endif /* end of __GNUC_PREREQ(4, 9) */
#endif /* end of defined(__GNUC_PREREQ) */

#if defined(BH_HAS_STD_ATOMIC) && !defined(__cplusplus)
#include <stdatomic.h>
#define os_memory_order_acquire memory_order_acquire
#define os_memory_order_release memory_order_release
#define os_memory_order_seq_cst memory_order_seq_cst
#define os_atomic_thread_fence atomic_thread_fence
#define os_atomic_cmpxchg atomic_compare_exchange_strong
#endif

#endif /* end of os_atomic_thread_fence */

/**
 * Initialize current thread environment if current thread
 * is created by developer but not runtime
 *
 * @return 0 if success, -1 otherwise
 */
// int os_thread_env_init(void) Only for Windows

/**
 * Destroy current thread environment
 */
// void os_thread_env_destroy(void) Only for Windows

/**
 * Whether the thread environment is initialized
 */
// bool os_thread_env_inited(void) Only for Windows

/**
 * Suspend execution of the calling thread for (at least)
 * usec microseconds
 *
 * @return 0 if success, -1 otherwise
 */
int os_usleep(uint32 usec)
{
    xila_thread_sleep_exact(usec);

    return 0;
}

/**
 * Creates a recursive mutex
 *
 * @param mutex [OUTPUT] pointer to mutex initialized.
 *
 * @return 0 if success
 */
int os_recursive_mutex_init(korp_mutex *mutex)
{
    if (xila_initialize_recursive_mutex(mutex))
        return 0;

    return -1;
}

/**
 * This function creates a condition variable
 *
 * @param cond [OUTPUT] pointer to condition variable
 *
 * @return 0 if success
 */
int os_cond_init(korp_cond *cond)
{
    return xila_condition_variable_new(cond);
}

/**
 * This function destroys condition variable
 *
 * @param cond pointer to condition variable
 *
 * @return 0 if success
 */
int os_cond_destroy(korp_cond *cond)
{
    return xila_condition_variable_remove(cond);
}

/**
 * Wait a condition variable.
 *
 * @param cond pointer to condition variable
 * @param mutex pointer to mutex to protect the condition variable
 *
 * @return 0 if success
 */
int os_cond_wait(korp_cond *cond, korp_mutex *mutex)
{
    return xila_condition_variable_wait(cond, mutex);
}

/**
 * Wait a condition varible or return if time specified passes.
 *
 * @param cond pointer to condition variable
 * @param mutex pointer to mutex to protect the condition variable
 * @param useconds microseconds to wait
 *
 * @return 0 if success
 */
int os_cond_reltimedwait(korp_cond *cond, korp_mutex *mutex, uint64 useconds)
{
    return xila_condition_variable_try_wait(cond, mutex, useconds);
}

/**
 * Signals the condition variable
 *
 * @param cond condition variable
 *
 * @return 0 if success
 */
int os_cond_signal(korp_cond *cond)
{
    return xila_condition_variable_signal(cond);
}

/**
 * Broadcast the condition variable
 *
 * @param cond condition variable
 *
 * @return 0 if success
 */
int os_cond_broadcast(korp_cond *cond)
{
    return xila_condition_variable_broadcast(cond);
}

/**
 * Initialize readwrite lock object
 *
 * @param cond [OUTPUT] pointer to a lock object variable
 *
 * @return 0 if success
 */
int os_rwlock_init(korp_rwlock *lock)
{
    return !xila_initialize_rwlock(lock);
}

/**
 * Acquire the read lock
 *
 * @param lock lock variable
 *
 * @return 0 if success
 */
int os_rwlock_rdlock(korp_rwlock *lock)
{
    return !xila_read_rwlock(lock);
}

/**
 * Acquire the write lock
 *
 * @param lock lock variable
 *
 * @return 0 if success
 */
int os_rwlock_wrlock(korp_rwlock *lock)
{
    return !xila_write_rwlock(lock);
}

/**
 * Unlocks the lock object
 *
 * @param lock lock variable
 *
 * @return 0 if success
 */
int os_rwlock_unlock(korp_rwlock *lock)
{
    return !xila_unlock_rwlock(lock);
}

/**
 * Destroy a lock object
 *
 * @param lock lock variable
 *
 * @return 0 if success
 */
int os_rwlock_destroy(korp_rwlock *lock)
{
    return !xila_destroy_rwlock(lock);
}

/**
 * Creates a new POSIX-like semaphore or opens an existing
 * semaphore.  The semaphore is identified by name.  For details of
 * the construction of name, please refer to
 * https://man7.org/linux/man-pages/man3/sem_open.3.html.
 *
 * @param name semaphore name
 * @param oflasg specifies flags that control the operation of the call
 * @param mode permission flags
 * @param val initial value of the named semaphore.
 *
 * @return korp_sem * if success, NULL otherwise
 */
korp_sem *
os_sem_open(const char *name, int oflags, int mode, int val)
{
    return xila_semaphore_open(name, oflags, mode, val);
}

/**
 * Closes the named semaphore referred to by sem,
 * allowing any resources that the system has allocated to the
 * calling process for this semaphore to be freed.
 *
 * @param sem
 *
 * @return 0 if success
 */
int os_sem_close(korp_sem *sem)
{
    return xila_semaphore_close(sem);
}

/**
 * Decrements (locks) the semaphore pointed to by sem.
 * If the semaphore's value is greater than zero, then the decrement
 * proceeds, and the function returns, immediately.  If the
 * semaphore currently has the value zero, then the call blocks
 * until either it becomes possible to perform the decrement (i.e.,
 * the semaphore value rises above zero), or a signal handler
 * interrupts the call.
 *
 * @return 0 if success
 */
int os_sem_wait(korp_sem *sem)
{
    return xila_semaphore_wait(sem);
}

/**
 * Is the same as sem_wait(), except that if the
 * decrement cannot be immediately performed, then call returns an
 * error (errno set to EAGAIN) instead of blocking.
 *
 * @return 0 if success
 */
int os_sem_trywait(korp_sem *sem)
{
    return xila_semaphore_try_wait(sem);
}

/**
 * Increments (unlocks) the semaphore pointed to by sem.
 * If the semaphore's value consequently becomes greater than zero,
 * then another process or thread blocked in a sem_wait(3) call will
 * be woken up and proceed to lock the semaphore.
 *
 * @return 0 if success
 */
int os_sem_post(korp_sem *sem)
{
    return xila_semaphore_post(sem);
}

/**
 * Places the current value of the semaphore pointed
 * to sem into the integer pointed to by sval.
 *
 * @return 0 if success
 */
int os_sem_getvalue(korp_sem *sem, int *sval)
{
    return xila_semaphore_get_value(sem, sval);
}

/**
 * Remove the named semaphore referred to by name.
 * The semaphore name is removed immediately.  The semaphore is
 * destroyed once all other processes that have the semaphore open
 * close it.
 *
 * @param name semaphore name
 *
 * @return 0 if success
 */
int os_sem_unlink(const char *name)
{
    return xila_semaphore_remove(name);
}

/**
 * Initialize process-global state for os_wakeup_blocking_op.
 */
int os_blocking_op_init()
{
    return 0;
}

/**
 * Start accepting os_wakeup_blocking_op requests for the calling thread.
 */
void os_begin_blocking_op()
{
    xila_thread_begin_blocking_operation();
}

/**
 * Stop accepting os_wakeup_blocking_op requests for the calling thread.
 */
void os_end_blocking_op()
{
    xila_thread_end_blocking_operation();
}

/**
 * Wake up the specified thread.
 *
 * For example, on posix-like platforms, this can be implemented by
 * sending a signal (w/o SA_RESTART) which interrupts a blocking
 * system call.
 */
int os_wakeup_blocking_op(korp_tid tid)
{
    return xila_thread_wake_up(tid);
}

/****************************************************
 *                     Section 2                    *
 *                   Socket support                 *
 ****************************************************/

/**
 * NOTES:
 * Socket APIs are required by source debugging feature.
 * If you don't need source debugging feature, then no
 * need to implement these APIs
 */

/**
 * Create a socket
 *
 * @param sock [OUTPUT] the pointer of socket
 * @param is_ipv4 true for IPv4, false for IPv6
 * @param is_tcp true for tcp, false for udp
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_create(bh_socket_t *sock, bool is_ipv4, bool is_tcp)
{
    return -1;
}

/**
 * Assign the address and port to the socket
 *
 * @param socket the socket to bind
 * @param addr the ip address, only IPv4 supported currently
 * @param port [INPUT/OUTPUT] the port number, if the value is 0,
 *             it will use a port assigned by OS. On return it will
 *             contain the actual bound port number
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_bind(bh_socket_t socket, const char *addr, int *port)
{
    return -1;
}

/**
 * Set timeout for the given socket
 *
 * @param socket the socket to set timeout
 * @param timeout_us timeout in microseconds
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_settimeout(bh_socket_t socket, uint64 timeout_us)
{
    return -1;
}

/**
 * Make the socket as a passive socket to accept incoming connection requests
 *
 * @param socket the socket to listen
 * @param max_client maximum clients
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_listen(bh_socket_t socket, int max_client)
{
    return -1;
}

/**
 * Accept an incoming connection
 *
 * @param server_sock the socket to accept new connections
 * @param sock [OUTPUT] the connected socket
 * @param addr [OUTPUT] the address of the peer socket. If addr is NULL,
 *             nothing is filled in, and addrlen will not be used
 * @param addrlen [INPUT/OUTPUT] the size (in bytes) of the structure
 *                pointed to by addr, on return it will contain the actual
 *                size of the peer address
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_accept(bh_socket_t server_sock, bh_socket_t *sock, void *addr,
                     unsigned int *addrlen)
{
    return -1;
}

/**
 * initiate a connection on a socket
 *
 * @param socket the socket to connect with
 * @param addr the ip address, only IPv4 supported currently
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_connect(bh_socket_t socket, const char *addr, int port)
{
    return -1;
}

/**
 * Blocking receive message from a socket.
 *
 * @param socket the socket to receive message from
 * @param buf the buffer to store the data
 * @param len length of the buffer, this API does not guarantee that
 *            [len] bytes are received
 *
 * @return number of bytes received if success, -1 otherwise
 */
int os_socket_recv(bh_socket_t socket, void *buf, unsigned int len)
{
    return -1;
}

/**
 * Blocking receive message from a socket.
 *
 * @param socket the socket to send message
 * @param buf the buffer to store the data
 * @param len length of the buffer, this API does not guarantee that
 *            [len] bytes are received
 * @param flags control the operation
 * @param src_addr source address
 *
 * @return number of bytes sent if success, -1 otherwise
 */
int os_socket_recv_from(bh_socket_t socket, void *buf, unsigned int len, int flags,
                        bh_sockaddr_t *src_addr)
{
    return -1;
}

/**
 * Blocking send message on a socket
 *
 * @param socket the socket to send message
 * @param buf the buffer of data to be sent
 * @param len length of the buffer
 *
 * @return number of bytes sent if success, -1 otherwise
 */
int os_socket_send(bh_socket_t socket, const void *buf, unsigned int len)
{
    return -1;
}

/**
 * Blocking send message on a socket to the target address
 *
 * @param socket the socket to send message
 * @param buf the buffer of data to be sent
 * @param len length of the buffer
 * @param flags control the operation
 * @param dest_addr target address
 *
 * @return number of bytes sent if success, -1 otherwise
 */
int os_socket_send_to(bh_socket_t socket, const void *buf, unsigned int len,
                      int flags, const bh_sockaddr_t *dest_addr)
{

    return -1;
}

/**
 * Close a socket
 *
 * @param socket the socket to be closed
 *
 * @return always return 0
 */
int os_socket_close(bh_socket_t socket)
{
    return -1;
}

/**
 * Shutdown a socket
 *
 * @param socket the socket to be shutdown
 *
 * @return returns corresponding error code
 */
__wasi_errno_t
os_socket_shutdown(bh_socket_t socket)
{
    return -1;
}

/**
 * converts cp into a number in host byte order suitable for use as
 * an Internet network address
 *
 * @param is_ipv4 a flag that indicates whether the string is an IPv4 or
 * IPv6 address
 *
 * @param cp a string in IPv4 numbers-and-dots notation or IPv6
 * numbers-and-colons notation
 *
 * @param out an output buffer to store binary address
 *
 * @return On success, the function returns 0.
 * If the input is invalid, -1 is returned
 */
int os_socket_inet_network(bool is_ipv4, const char *cp, bh_ip_addr_buffer_t *out)
{
    return -1;
}

/**
 * Resolve a host a hostname and a service to one or more IP addresses
 *
 * @param host a host to resolve
 *
 * @param service a service to find a port for
 *
 * @param hint_is_tcp an optional flag that determines a preferred socket type
 (TCP or UDP).
 *
 * @param hint_is_ipv4 an optional flag that determines a preferred address
 family (IPv4 or IPv6)
 *
 * @param addr_info a buffer for resolved addresses
 *
 * @param addr_info_size a size of the buffer for resolved addresses

 * @param max_info_size a maximum number of addresses available (can be bigger
 or smaller than buffer size)

 * @return On success, the function returns 0; otherwise, it returns -1
 */
int os_socket_addr_resolve(const char *host, const char *service,
                           uint8_t *hint_is_tcp, uint8_t *hint_is_ipv4,
                           bh_addr_info_t *addr_info, size_t addr_info_size,
                           size_t *max_info_size)
{
    return -1;
}

/**
 * Returns an binary address and a port of the local socket
 *
 * @param socket the local socket
 *
 * @param sockaddr a buffer for storing the address
 *
 * @return On success, returns 0; otherwise, it returns -1.
 */
int os_socket_addr_local(bh_socket_t socket, bh_sockaddr_t *sockaddr)
{
    return -1;
}

/**
 * Returns an binary address and a port of the remote socket
 *
 * @param socket the remote socket
 *
 * @param sockaddr a buffer for storing the address
 *
 * @return On success, returns 0; otherwise, it returns -1.
 */
int os_socket_addr_remote(bh_socket_t socket, bh_sockaddr_t *sockaddr)
{
    return -1;
}

/**
 * Set the maximum send buffer size.
 *
 * @param socket the socket to set
 * @param bufsiz requested kernel buffer size
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_send_buf_size(bh_socket_t socket, size_t bufsiz)
{
    return -1;
}

/**
 * Get the maximum send buffer size.
 *
 * @param socket the socket to set
 * @param bufsiz the returned kernel buffer size
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_get_send_buf_size(bh_socket_t socket, size_t *bufsiz)
{
    return -1;
}

/**
 * Set the maximum receive buffer size.
 *
 * @param socket the socket to set
 * @param bufsiz requested kernel buffer size
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_recv_buf_size(bh_socket_t socket, size_t bufsiz)
{
    return -1;
}

/**
 * Get the maximum receive buffer size.
 *
 * @param socket the socket to set
 * @param bufsiz the returned kernel buffer size
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_get_recv_buf_size(bh_socket_t socket, size_t *bufsiz)
{
    return -1;
}

/**
 * Enable sending of keep-alive messages on connection-oriented sockets
 *
 * @param socket the socket to set the flag
 * @param is_enabled 1 to enable or 0 to disable
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_keep_alive(bh_socket_t socket, bool is_enabled)
{
    return -1;
}

/**
 * Get if sending of keep-alive messages on connection-oriented sockets is
 * enabled
 *
 * @param socket the socket to check
 * @param is_enabled 1 if enabled or 0 if disabled
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_get_keep_alive(bh_socket_t socket, bool *is_enabled)
{
    return -1;
}

/**
 * Set the send timeout until reporting an error
 *
 * @param socket the socket to set
 * @param time_us microseconds until timeout
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_send_timeout(bh_socket_t socket, uint64 timeout_us)
{
    return -1;
}

/**
 * Get the send timeout until reporting an error
 *
 * @param socket the socket to set
 * @param time_us the returned microseconds until timeout
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_get_send_timeout(bh_socket_t socket, uint64 *timeout_us)
{
    return -1;
}

/**
 * Set the recv timeout until reporting an error
 *
 * @param socket the socket to set
 * @param time_us microseconds until timeout
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_recv_timeout(bh_socket_t socket, uint64 timeout_us)
{
    return -1;
}

/**
 * Get the recv timeout until reporting an error
 *
 * @param socket the socket to set
 * @param time_us the returned microseconds until timeout
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_get_recv_timeout(bh_socket_t socket, uint64 *timeout_us)
{
    return -1;
}

/**
 * Enable re-use of local addresses
 *
 * @param socket the socket to set
 * @param is_enabled 1 to enable or 0 to disable
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_reuse_addr(bh_socket_t socket, bool is_enabled)
{
    return -1;
}

/**
 * Get whether re-use of local addresses is enabled
 *
 * @param socket the socket to set
 * @param is_enabled 1 for enabled or 0 for disabled
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_get_reuse_addr(bh_socket_t socket, bool *is_enabled)
{
    return -1;
}

/**
 * Enable re-use of local ports
 *
 * @param socket the socket to set
 * @param is_enabled 1 to enable or 0 to disable
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_reuse_port(bh_socket_t socket, bool is_enabled)
{
    return -1;
}

/**
 * Get whether re-use of local ports is enabled
 *
 * @param socket the socket to set
 * @param is_enabled 1 for enabled or 0 for disabled
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_get_reuse_port(bh_socket_t socket, bool *is_enabled)
{
    return -1;
}

/**
 * Set the linger options for the given socket
 *
 * @param socket the socket to set
 * @param is_enabled whether linger is enabled
 * @param linger_s linger time (seconds)
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_linger(bh_socket_t socket, bool is_enabled, int linger_s)
{
    return -1;
}

/**
 * Get the linger options for the given socket
 *
 * @param socket the socket to get
 * @param is_enabled whether linger is enabled
 * @param linger_s linger time (seconds)
 * @return 0 if success, -1 otherwise
 */
int os_socket_get_linger(bh_socket_t socket, bool *is_enabled, int *linger_s)
{
    return -1;
}

/**
 * Set no delay TCP
 * If set, disable the Nagle algorithm.
 * This means that segments are always sent as soon as possible,
 * even if there is only a small amount of data
 *
 * @param socket the socket to set the flag
 * @param is_enabled 1 to enable or 0 to disable
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_tcp_no_delay(bh_socket_t socket, bool is_enabled)
{
    return -1;
}

/**
 * Get no delay TCP
 * If set, disable the Nagle algorithm.
 * This means that segments are always sent as soon as possible,
 * even if there is only a small amount of data
 *
 * @param socket the socket to check
 * @param is_enabled 1 if enabled or 0 if disabled
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_get_tcp_no_delay(bh_socket_t socket, bool *is_enabled)
{
    return -1;
}

/**
 * Enable/Disable tcp quickack mode
 * In quickack mode, acks are sent immediately, rather than delayed if needed in
 * accordance to normal TCP operation
 *
 * @param socket the socket to set the flag
 * @param is_enabled 1 to enable or 0 to disable
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_tcp_quick_ack(bh_socket_t socket, bool is_enabled)
{
    return -1;
}

/**
 * Enable/Disable tcp quickack mode
 * In quickack mode, acks are sent immediately, rather than delayed if needed in
 * accordance to normal TCP operation
 *
 * @param socket the socket to check
 * @param is_enabled 1 if enabled or 0 if disabled
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_get_tcp_quick_ack(bh_socket_t socket, bool *is_enabled)
{
    return -1;
}

/**
 * Set the time the connection needs to remain idle before sending keepalive
 * probes
 *
 * @param socket the socket to set
 * @param time_s seconds until keepalive probes are sent
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_tcp_keep_idle(bh_socket_t socket, uint32_t time_s)
{
    return -1;
}

/**
 * Gets the time the connection needs to remain idle before sending keepalive
 * probes
 *
 * @param socket the socket to check
 * @param time_s seconds until keepalive probes are sent
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_get_tcp_keep_idle(bh_socket_t socket, uint32_t *time_s)
{
    return -1;
}

/**
 * Set the time between individual keepalive probes
 *
 * @param socket the socket to set
 * @param time_us seconds between individual keepalive probes
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_tcp_keep_intvl(bh_socket_t socket, uint32_t time_s)
{
    return -1;
}

/**
 * Get the time between individual keepalive probes
 *
 * @param socket the socket to get
 * @param time_s seconds between individual keepalive probes
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_get_tcp_keep_intvl(bh_socket_t socket, uint32_t *time_s)
{
    return -1;
}

/**
 * Set use of TCP Fast Open
 *
 * @param socket the socket to set
 * @param is_enabled 1 to enable or 0 to disable
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_tcp_fastopen_connect(bh_socket_t socket, bool is_enabled)
{
    return -1;
}

/**
 * Get whether use of TCP Fast Open is enabled
 *
 * @param socket the socket to get
 * @param is_enabled 1 to enabled or 0 to disabled
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_get_tcp_fastopen_connect(bh_socket_t socket, bool *is_enabled)
{
    return -1;
}

/**
 * Set enable or disable IPv4 or IPv6 multicast loopback.
 *
 * @param socket the socket to set
 * @param ipv6 true to set ipv6 loopback or false for ipv4
 * @param is_enabled 1 to enable or 0 to disable
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_ip_multicast_loop(bh_socket_t socket, bool ipv6, bool is_enabled)
{
    return -1;
}

/**
 * Get enable or disable IPv4 or IPv6 multicast loopback.
 *
 * @param socket the socket to check
 * @param ipv6 true to set ipv6 loopback or false for ipv4
 * @param is_enabled 1 for enabled or 0 for disabled
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_get_ip_multicast_loop(bh_socket_t socket, bool ipv6,
                                    bool *is_enabled)
{
    return -1;
}

/**
 * Add membership to a group
 *
 * @param socket the socket to add membership to
 * @param imr_multiaddr the group multicast address (IPv4 or IPv6)
 * @param imr_interface the interface to join on
 * @param is_ipv6 whether the imr_multiaddr is IPv4 or IPv6 (true for IPv6)
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_ip_add_membership(bh_socket_t socket,
                                    bh_ip_addr_buffer_t *imr_multiaddr,
                                    uint32_t imr_interface, bool is_ipv6)
{
    return -1;
}

/**
 * Drop membership of a group
 *
 * @param socket the socket to drop membership to
 * @param imr_multiaddr the group multicast address (IPv4 or IPv6)
 * @param imr_interface the interface to join on
 * @param is_ipv6 whether the imr_multiaddr is IPv4 or IPv6 (true for IPv6)
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_ip_drop_membership(bh_socket_t socket,
                                     bh_ip_addr_buffer_t *imr_multiaddr,
                                     uint32_t imr_interface, bool is_ipv6)
{
    return -1;
}

/**
 * Set the current time-to-live field that is
 * used in every packet sent from this socket.
 * @param socket the socket to set the flag
 * @param ttl_s time to live (seconds)
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_ip_ttl(bh_socket_t socket, uint8_t ttl_s)
{
    return -1;
}

/**
 * Retrieve the current time-to-live field that is
 * used in every packet sent from this socket.
 * @param socket the socket to set the flag
 * @param ttl_s time to live (seconds)
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_get_ip_ttl(bh_socket_t socket, uint8_t *ttl_s)
{
    return -1;
}

/**
 * Set the time-to-live value of outgoing multicast
 * packets for this socket
 * @param socket the socket to set the flag
 * @param ttl_s time to live (seconds)
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_ip_multicast_ttl(bh_socket_t socket, uint8_t ttl_s)
{
    return -1;
}

/**
 * Read the time-to-live value of outgoing multicast
 * packets for this socket
 * @param socket the socket to set the flag
 * @param ttl_s time to live (seconds)
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_get_ip_multicast_ttl(bh_socket_t socket, uint8_t *ttl_s)
{
    return -1;
}

/**
 * Restrict to sending and receiving IPv6 packets only
 *
 * @param socket the socket to set
 * @param is_enabled 1 to enable or 0 to disable
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_ipv6_only(bh_socket_t socket, bool is_enabled)
{
    return -1;
}

/**
 * Get whether only sending and receiving IPv6 packets
 *
 * @param socket the socket to check
 * @param is_enabled 1 for enabled or 0 for disabled
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_get_ipv6_only(bh_socket_t socket, bool *is_enabled)
{
    return -1;
}

/**
 * Set whether broadcast is enabled
 * When enabled, datagram sockets are allowed
 * to send packets to a broadcast address.
 *
 * @param socket the socket to set the flag
 * @param is_enabled 1 to enable or 0 to disable
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_set_broadcast(bh_socket_t socket, bool is_enabled)
{
    return -1;
}

/**
 * Get whether broadcast is enabled
 * When enabled, datagram sockets are allowed
 * to send packets to a broadcast address.
 *
 * @param socket the socket to check
 * @param is_enabled 1 if enabled or 0 if disabled
 *
 * @return 0 if success, -1 otherwise
 */
int os_socket_get_broadcast(bh_socket_t socket, bool *is_enabled)
{
    return -1;
}

/**
 * Dump memory information of the current process
 * It may have variant implementations in different platforms
 *
 * @param out the output buffer. It is for sure the return content
 *            is a c-string which ends up with '\0'
 * @param size the size of the output buffer
 *
 * @return 0 if success, -1 otherwise
 */
int os_dumps_proc_mem_info(char *out, unsigned int size)
{
    return -1;
}

/****************************************************
 *                     Section 3                    *
 *                 Filesystem support               *
 ****************************************************/

/**
 * Returns an invalid handle that is guaranteed to cause failure when
 * called with any filesystem operation.
 *
 * @return the invalid handle
 */
os_file_handle os_get_invalid_handle()
{
    return 0xFFFFFFFFFFFFFFFF;
}

os_raw_file_handle
os_invalid_raw_handle()
{
    return 0xFFFFFFFFFFFFFFFF;
}


/**
 * NOTES:
 * Fileystem APIs are required for WASI libc support. If you don't need to
 * support WASI libc, there is no need to implement these APIs. With a
 * few exceptions, each filesystem function has been named after the equivalent
 * POSIX filesystem function with an os_ prefix.
 *
 * Filesystem types
 *
 * os_raw_file_handle: the underlying OS file handle type e.g. int on POSIX
 * systems and HANDLE on Windows. This type exists to allow embedders to provide
 * custom file handles for stdout/stdin/stderr.
 *
 * os_file_handle: the file handle type used in the WASI libc fd
 * table. Filesystem implementations can use it as a means to store any
 * necessary platform-specific information which may not be directly available
 * through the raw OS file handle. Similiar to POSIX file descriptors, file
 * handles may also refer to sockets, directories, symbolic links or character
 * devices and any of the filesystem operations which make sense for these
 * resource types should be supported as far as possible.
 *
 * os_dir_stream: a directory stream type in which fileystem implementations
 * can store any necessary state to iterate over the entries in a directory.
 */

/**
 * Obtain information about an open file associated with the given handle.
 *
 * @param handle the handle for which to obtain file information
 * @param buf a buffer in which to store the information
 */
__wasi_errno_t
os_fstat(os_file_handle handle, struct __wasi_filestat_t *buf)
{
    XilaFileSystemStatistics file_system_statistics;

    XilaFileSystemResult file_system_result = xila_file_system_get_statistics(handle, &file_system_statistics);

    if (file_system_result == 0)
        into_wasi_file_statistics(&file_system_statistics, buf);

    return into_wasi_error(file_system_result);
}

/**
 * Obtain information about an open file or directory.
 * @param handle the directory handle from which to resolve the file/directory
 * path
 * @param path the relative path of the file or directory for which to obtain
 * information
 * @param buf a buffer in which to store the information
 * @param follow_symlink whether to follow symlinks when resolving the path
 */
__wasi_errno_t
os_fstatat(os_file_handle handle, const char *path,
           struct __wasi_filestat_t *buf, __wasi_lookupflags_t lookup_flags)
{
    bool follow_symlink = lookup_flags & __WASI_LOOKUP_SYMLINK_FOLLOW;

    XilaFileSystemStatistics file_system_statistics;

    XilaFileSystemResult file_system_result = xila_file_system_get_statistics_from_path(path, &file_system_statistics, follow_symlink);

    return into_wasi_error(file_system_result);
}

/**
 * Obtain the file status flags for the provided handle. This is similiar to the
 * POSIX function fcntl called with the F_GETFL command.
 *
 * @param handle the handle for which to obtain the file status flags
 * @param flags a pointer in which to store the output
 */
__wasi_errno_t
os_file_get_fdflags(os_file_handle handle, __wasi_fdflags_t *flags)
{
    return xila_file_system_get_flags(handle, flags);
}

/**
 * Set the file status flags for the provided handle. This is similiar to the
 * POSIX function fcntl called with the F_SETFL command.
 *
 * @param handle the handle for which to set the file status flags
 * @param flags the flags to set
 */
__wasi_errno_t
os_file_set_fdflags(os_file_handle handle, __wasi_fdflags_t flags)
{
    return xila_file_system_set_flags(handle, flags);
}

/**
 * Synchronize the data of a file to disk.
 *
 * @param handle
 */
__wasi_errno_t
os_fdatasync(os_file_handle handle)
{
    return into_wasi_error(xila_file_system_flush(handle, false));
}

/**
 * Synchronize the data and metadata of a file to disk.
 *
 * @param handle
 */
__wasi_errno_t
os_fsync(os_file_handle handle)
{
    return into_wasi_error(xila_file_system_flush(handle, true));
}

/**
 * Open a preopen directory. The path provided must refer to a directory and the
 * returned handle will allow only readonly operations.
 *
 * @param path the path of the preopen directory to open
 * @param out a pointer in which to store the newly opened handle
 */
__wasi_errno_t
os_open_preopendir(const char *path, os_file_handle *out)
{
    XilaFileSystemResult result = xila_file_system_open_directory(path, out);

    return into_wasi_error(result);
}

/**
 * Open a file or directory at the given path.
 *
 * @param handle a handle to the directory in which to open the new file or
 * directory
 * @param path the relative path of the file or directory to open
 * @param oflags the flags to determine how the file or directory is opened
 * @param fd_flags the flags to set on the returned handle
 * @param lookup_flags whether to follow symlinks when resolving the path
 * @param access_mode whether the file is opened as read only, write only or
 * both
 * @param out a pointer in which to store the newly opened handle
 */
__wasi_errno_t
os_openat(os_file_handle handle, const char *path, __wasi_oflags_t oflags,
          __wasi_fdflags_t fd_flags, __wasi_lookupflags_t lookup_flags,
          wasi_libc_file_access_mode access_mode, os_file_handle *out)
{
    if (oflags & __WASI_O_DIRECTORY)
    {
        if (path[0] == '.')
        {
            size_t path_size = strlen(path) + 1;
            char new_path[path_size];
            strncpy(new_path, path, path_size);
            new_path[0] = '/';

            return into_wasi_error(xila_file_system_open_directory(new_path, out));
        }
        else
        {
            return into_wasi_error(xila_file_system_open_directory(path, out));
        }
    }
    else
    {
        XilaFileSystemMode mode = into_xila_mode(access_mode);
        XilaFileSystemOpen open = into_xila_open(oflags);
        XilaFileSystemStatus status = into_xila_status(fd_flags);

        if (path[0] != '/')
        {
            size_t path_size = strlen(path) + 1;
            char new_path[path_size];
            strncpy(new_path + 1, path, path_size);
            new_path[0] = '/';

            return into_wasi_error(xila_file_system_open(new_path, mode, open, status, out));
        }

        return into_wasi_error(xila_file_system_open(path, mode, open, status, out));
    }
}

/**
 * Obtain the file access mode for the provided handle. This is similiar to the
 * POSIX function fcntl called with the F_GETFL command combined with the
 * O_ACCMODE mask.
 *
 * @param handle the handle for which to obtain the access mode
 * @param access_mode a pointer in which to store the access mode
 */
__wasi_errno_t
os_file_get_access_mode(os_file_handle handle,
                        wasi_libc_file_access_mode *access_mode)
{
    uint8_t mode;

    XilaFileSystemResult file_system_result = xila_file_system_get_access_mode(handle, &mode);

    if (file_system_result == 0)
    {
        *access_mode = into_wasi_access_mode(mode);
    }

    return into_wasi_error(file_system_result);
}

/**
 * Close the provided handle. If is_stdio is true, the raw file handle
 * associated with the given file handle will not be closed.
 *
 * @param handle the handle to close
 * @param is_stdio whether the provided handle refers to a stdio device
 */
__wasi_errno_t
os_close(os_file_handle handle, bool is_stdio)
{
    return into_wasi_error(xila_file_system_close(handle));
}

/**
 * Read data from the provided handle at the given offset into multiple buffers.
 *
 * @param handle the handle to read from
 * @param iov the buffers to read into
 * @param iovcnt the number of buffers to read into
 * @param offset the offset to read from
 * @param nread a pointer in which to store the number of bytes read
 */
__wasi_errno_t
os_preadv(os_file_handle handle, const struct __wasi_iovec_t *iov, int iovcnt,
          __wasi_filesize_t offset, size_t *nread)
{
    uint8_t *buffers[iovcnt];
    size_t lengths[iovcnt];

    for (int i = 0; i < iovcnt; i++)
    {
        buffers[i] = iov[i].buf;
        lengths[i] = iov[i].buf_len;
    }

    XilaFileSystemResult file_system_result = xila_file_system_read_at_position_vectored(handle, buffers, lengths, iovcnt, offset, nread);

    return into_wasi_error(file_system_result);
}

/**
 * Write data from multiple buffers at the given offset to the provided handle.
 *
 * @param handle the handle to write to
 * @param iov the buffers to write from
 * @param iovcnt the number of buffers to write from
 * @param offset the offset to write from
 * @param nwritten a pointer in which to store the number of bytes written
 */
__wasi_errno_t
os_pwritev(os_file_handle handle, const struct __wasi_ciovec_t *iov, int iovcnt,
           __wasi_filesize_t offset, size_t *nwritten)
{
    const uint8_t *buffers[iovcnt];
    size_t lengths[iovcnt];

    for (int i = 0; i < iovcnt; i++)
    {
        buffers[i] = iov[i].buf;
        lengths[i] = iov[i].buf_len;
    }

    XilaFileSystemResult file_system_result = xila_file_system_write_at_position_vectored(handle, buffers, lengths, iovcnt, offset, nwritten);

    return into_wasi_error(file_system_result);
}

/**
 * Read data from the provided handle into multiple buffers.
 *
 * @param handle the handle to read from
 * @param iov the buffers to read into
 * @param iovcnt the number of buffers to read into
 * @param nread a pointer in which to store the number of bytes read
 */
__wasi_errno_t
os_readv(os_file_handle handle, const struct __wasi_iovec_t *iov, int iovcnt,
         size_t *nread)
{
    uint8_t *buffers[iovcnt];
    size_t lengths[iovcnt];

    for (int i = 0; i < iovcnt; i++)
    {
        buffers[i] = iov[i].buf;
        lengths[i] = iov[i].buf_len;
    }

    return xila_file_system_read_vectored(handle, buffers, lengths, iovcnt, nread);
}

/**
 * Write data from multiple buffers to the provided handle.
 *
 * @param handle the handle to write to
 * @param iov the buffers to write from
 * @param iovcnt the number of buffers to write from
 * @param nwritten a pointer in which to store the number of bytes written
 */
__wasi_errno_t
os_writev(os_file_handle handle, const struct __wasi_ciovec_t *iov, int iovcnt,
          size_t *nwritten)
{
    const uint8_t *buffers[iovcnt];
    size_t lengths[iovcnt];

    for (int i = 0; i < iovcnt; i++)
    {
        buffers[i] = iov[i].buf;
        lengths[i] = iov[i].buf_len;
    }

    return xila_file_system_write_vectored(handle, buffers, lengths, iovcnt, nwritten);
}

/**
 * Allocate storage space for the file associated with the provided handle. This
 * is similar to the POSIX function posix_fallocate.
 *
 * @param handle the handle to allocate space for
 * @param offset the offset to allocate space at
 * @param length the amount of space to allocate
 */
__wasi_errno_t
os_fallocate(os_file_handle handle, __wasi_filesize_t offset,
             __wasi_filesize_t length)
{
    return xila_file_system_allocate(handle, offset, length);
}

/**
 * Adjust the size of an open file.
 *
 * @param handle the associated file handle for which to adjust the size
 * @param size the new size of the file
 */
__wasi_errno_t
os_ftruncate(os_file_handle handle, __wasi_filesize_t size)
{
    return xila_file_system_truncate(handle, size);
}

/**
 * Set file access and modification times on an open file or directory.
 *
 * @param handle the associated file handle for which to adjust the
 * access/modification times
 * @param access_time the timestamp for the new access time
 * @param modification_time the timestamp for the new modification time
 * @param fstflags a bitmask to indicate which timestamps to adjust
 */
__wasi_errno_t
os_futimens(os_file_handle handle, __wasi_timestamp_t access_time,
            __wasi_timestamp_t modification_time, __wasi_fstflags_t fstflags)
{
    return xila_file_system_set_times(handle, access_time, modification_time, fstflags);
}

/**
 * Set file access and modification times on an open file or directory.
 *
 * @param handle the directory handle from which to resolve the path
 * @param path the relative path of the file or directory for which to adjust
 * the access/modification times
 * @param access_time the timestamp for the new access time
 * @param modification_time the timestamp for the new modification time
 * @param fstflags a bitmask to indicate which timestamps to adjust
 * @param lookup_flags whether to follow symlinks when resolving the path
 */
__wasi_errno_t
os_utimensat(os_file_handle handle, const char *path,
             __wasi_timestamp_t access_time,
             __wasi_timestamp_t modification_time, __wasi_fstflags_t fstflags,
             __wasi_lookupflags_t lookup_flags)
{
    bool Follow = lookup_flags & __WASI_LOOKUP_SYMLINK_FOLLOW;

    return xila_file_system_set_times_from_path(path, access_time, modification_time,
                                                (uint8_t)fstflags, Follow);
}

/**
 * Read the contents of a symbolic link relative to the provided directory
 * handle.
 *
 * @param handle the directory handle
 * @param path the relative path of the symbolic link from which to read
 * @param buf the buffer to read the link contents into
 * @param bufsize the size of the provided buffer
 * @param nread a pointer in which to store the number of bytes read into the
 * buffer
 */
__wasi_errno_t
os_readlinkat(os_file_handle handle, const char *path, char *buf,
              size_t bufsize, size_t *nread)
{
    return __WASI_EINVAL;
}

/**
 * Create a link from one path to another path.
 *
 * @param from_handle the directory handle from which to resolve the origin path
 * @param from_path the origin path to link from
 * @param to_handle the directory handle from which to resolve the destination
 * path
 * @param to_path the destination path at which to create the link
 * @param lookup_flags whether to follow symlinks when resolving the origin path
 */
__wasi_errno_t
os_linkat(os_file_handle from_handle, const char *from_path,
          os_file_handle to_handle, const char *to_path,
          __wasi_lookupflags_t lookup_flags)
{
    return into_wasi_error(
        xila_file_system_link(from_path, to_path));
}

/**
 * Create a symbolic link from one path to another path.
 *
 * @param old_path the symbolic link contents
 * @param handle the directory handle from which to resolve the destination path
 * @param new_path the destination path at which to create the symbolic link
 */
__wasi_errno_t
os_symlinkat(const char *old_path, os_file_handle handle, const char *new_path)
{
    return into_wasi_error(xila_file_system_create_symbolic_link_at(handle, old_path, new_path));
}

/**
 * Create a directory relative to the provided directory handle.
 *
 * @param handle the directory handle
 * @param path the relative path of the directory to create
 */
__wasi_errno_t
os_mkdirat(os_file_handle handle, const char *path)
{
    if (path[0] != '/')
    {
        size_t path_size = strlen(path) + 2;
        char new_path[strlen(path) + 2];

        new_path[0] = '/';
        strncpy(new_path + 1, path, path_size);

        return into_wasi_error(xila_file_system_create_directory(new_path));
    }

    return into_wasi_error(xila_file_system_create_directory(path));
}

/**
 * Rename a file or directory.
 *
 * @param old_handle the directory handle from which to resolve the old path
 * @param old_path the source path to rename
 * @param new_handle the directory handle from which to resolve the destination
 * path
 * @param new_path the destination path to which to rename the file or directory
 */
__wasi_errno_t
os_renameat(os_file_handle old_handle, const char *old_path,
            os_file_handle new_handle, const char *new_path)
{
    size_t old_path_size = strlen(old_path) + 2;

    char old_new_path[old_path_size];

    old_new_path[0] = '/';
    strncpy(old_new_path + 1, old_path, old_path_size);

    size_t new_path_size = strlen(new_path) + 2;

    char new_new_path[new_path_size];

    new_new_path[0] = '/';
    strncpy(new_new_path + 1, new_path, new_path_size);

    return into_wasi_error(xila_file_system_rename(old_new_path, new_new_path));
}

/**
 * Unlink a file or directory.
 *
 * @param handle the directory handle from which to resolve the path
 * @param path the relative path of the file or directory to unlink
 * @param is_dir whether the provided handle refers to a directory or file
 */
__wasi_errno_t
os_unlinkat(os_file_handle handle, const char *path, bool is_dir)
{
    return xila_file_system_remove(path);
}

/**
 * Move the read/write offset of an open file.
 *
 * @param handle the associated file handle for which to adjust the offset
 * @param offset the number of bytes to adjust the offset by
 * @param whence the position whence to adjust the offset
 * @param new_offset a pointer in which to store the new offset
 */
__wasi_errno_t
os_lseek(os_file_handle handle, __wasi_filedelta_t offset,
         __wasi_whence_t whence, __wasi_filesize_t *new_offset)
{
    XilaFileSystemWhence Whence = into_xila_whence(whence);

    return into_wasi_error(xila_file_system_set_position(handle, offset, Whence, new_offset));
}

/**
 * Provide file advisory information for the given handle. This is similar to
 * the POSIX function posix_fadvise.
 *
 * @param handle the associated file handle for which to provide advisory
 * information
 * @param offset the offset within the file to which the advisory
 * information applies
 * @param length the length of the region for which the advisory information
 * applies
 * @param advice the advice to provide
 */
__wasi_errno_t
os_fadvise(os_file_handle handle, __wasi_filesize_t offset,
           __wasi_filesize_t length, __wasi_advice_t advice)
{
    return xila_file_system_advise(handle, offset, length, advice);
}

/**
 * Determine if the given handle refers to a terminal device. __WASI_ESUCCESS
 * will be returned if the handle is associated with a terminal device,
 * otherwise an appropriate error code will be returned.
 *
 * @param handle
 */
__wasi_errno_t
os_isatty(os_file_handle handle)
{
    bool Is_Terminal = false;

    XilaFileSystemResult Result = xila_file_system_is_a_terminal(handle, &Is_Terminal);

    if (Is_Terminal)
    {
        return __WASI_ESUCCESS;
    }

    return into_wasi_error(Result);
}

/**
 * Converts a raw file handle to STDIN to a corresponding file handle to STDIN.
 * If the provided raw file handle is invalid, the platform-default raw handle
 * for STDIN will be used.
 *
 * @param raw_stdin a raw file handle to STDIN
 *
 * @return a handle to STDIN
 */
os_file_handle
os_convert_stdin_handle(os_raw_file_handle raw_stdin)
{
    return raw_stdin;
}

/**
 * Converts a raw file handle to STDOUT to a correponding file handle to STDOUT.
 * If the provided raw file handle is invalid, the platform-default raw handle
 * for STDOUT will be used.
 *
 * @param raw_stdout a raw file handle to STDOUT
 *
 * @return a handle to STDOUT
 */
os_file_handle
os_convert_stdout_handle(os_raw_file_handle raw_stdout)
{
    return raw_stdout;
}

/**
 * Converts a raw file handle to STDERR to a correponding file handle to STDERR.
 * If the provided raw file handle is invalid, the platform-default raw handle
 * for STDERR will be used.
 *
 * @param raw_stderr a raw file handle to STDERR
 *
 * @return a handle to STDERR
 */
os_file_handle
os_convert_stderr_handle(os_raw_file_handle raw_stderr)
{
    return raw_stderr;
}

/**
 * Open a directory stream for the provided directory handle. The returned
 * directory stream will be positioned at the first entry in the directory.
 *
 * @param handle the directory handle
 * @param dir_stream a pointer in which to store the new directory stream
 */
__wasi_errno_t
os_fdopendir(os_file_handle handle, os_dir_stream *dir_stream)
{
    *dir_stream = handle;

    return __WASI_ESUCCESS;
}

/**
 * Reset the position of a directory stream to the beginning of the directory.
 *
 * @param dir_stream the directory stream for which to reset the position
 */
__wasi_errno_t
os_rewinddir(os_dir_stream dir_stream)
{
    return into_wasi_error(xila_file_system_rewind_directory(dir_stream));
}

/**
 * Set the position of the given directory stream.
 *
 * @param dir_stream the directory stream for which to set the position
 * @param position the position to set
 */
__wasi_errno_t
os_seekdir(os_dir_stream dir_stream, __wasi_dircookie_t position)
{
    XilaFileSystemResult Result = xila_file_system_directory_set_position(dir_stream, position);

    return into_wasi_error(Result);
}

/**
 * Read a directory entry from the given directory stream. The directory name
 * will be NULL if the end of the directory is reached or an error is
 * encountered.
 *
 * @param dir_stream the directory stream from which to read the entry
 * @param entry a pointer in which to store the directory entry
 * @param d_name a pointer in which to store the directory entry name
 */
__wasi_errno_t os_readdir(os_dir_stream dir_stream, __wasi_dirent_t *entry, const char **d_name)
{
    XilaFileSystemSize size = 0;
    XilaFileSystemInode inode = 0;
    XilaFileKind type = 0;

    XilaFileSystemResult file_system_result = xila_file_system_read_directory(dir_stream, d_name, &type, &size, &inode);

    if ((*d_name) != NULL)
    {
        entry->d_ino = inode;
        entry->d_namlen = strlen(*d_name);
        entry->d_type = into_wasi_file_type(type);
    }
    return into_wasi_error(file_system_result);
}

/**
 * Close the given directory stream. The handle associated with the directory
 * stream will also be closed.
 *
 * @param dir_stream the directory stream to close
 */
__wasi_errno_t
os_closedir(os_dir_stream dir_stream)
{
    return into_wasi_error(xila_file_system_close_directory(dir_stream));
}

/**
 * Returns an invalid directory stream that is guaranteed to cause failure when
 * called with any directory filesystem operation.
 *
 * @return the invalid directory stream
 */
os_dir_stream os_get_invalid_dir_stream()
{
    return 0xFFFFFFFFFFFFFFFF;
}

/**
 * Checks whether the given directory stream is valid. An invalid directory
 * stream is guaranteed to cause failure when called with any directory
 * filesystem operation.
 *
 * @param dir_stream a pointer to a directory stream
 */
bool os_is_dir_stream_valid(os_dir_stream *dir_stream)
{
    return *dir_stream != os_get_invalid_dir_stream();
}

/**
 * Checks whether the given file handle is valid. An invalid handle is
 * guaranteed to cause failure when called with any filesystem operation.
 *
 * @param handle a pointer to a file handle
 */
bool os_is_handle_valid(os_file_handle *handle)
{
    return *handle != os_get_invalid_handle();
}

/**
 * Resolve a pathname. The generated pathname will be stored as a
 * null-terminated string, with a maximum length of PATH_MAX bytes.
 *
 * @param path the path to resolve
 * @param resolved_path the buffer to store the resolved path in
 *
 * @return the resolved path if success, NULL otherwise
 */
char *os_realpath(const char *path, char *resolved_path)
{
 
    // XilaFileSystemResult Result = xila_file_system_resolve_path(path, resolved_path, PATH_MAX);

    // printf("os_realpath: %s - %u\n", resolved_path, Result);

    // if (Result == 0)
    //     return resolved_path;
    // else
    //     return NULL;

    strncpy(resolved_path, path, PATH_MAX);

    return resolved_path;
}

/****************************************************
 *                     Section 4                    *
 *                  Clock functions                 *
 ****************************************************/

/**
 * NOTES:
 * Clock functions are required for WASI libc support. If you don't need to
 * support WASI libc, there is no need to implement these APIs.
 */

/**
 * Get the resolution of the specified clock.
 *
 * @param clock_id clock identifier
 * @param resolution output variable to store the clock resolution
 */
__wasi_errno_t os_clock_res_get(__wasi_clockid_t clock_id, __wasi_timestamp_t *resolution)
{
    return xila_time_get_resolution(clock_id, resolution);
}

/**
 * Get the current time of the specified clock.
 *
 * @param clock_id clock identifier
 * @param precision the maximum lag that the returned time value may have,
 * compared to its actual value.
 * @param time output variable to store the clock time
 */
__wasi_errno_t os_clock_time_get(__wasi_clockid_t clock_id, __wasi_timestamp_t precision,
                                 __wasi_timestamp_t *time)
{
    return xila_time_get_time(clock_id, precision, time);
}

bool os_is_stdin_handle(os_file_handle fd)
{
    return xila_file_system_is_stdin(fd);
}

bool os_is_stdout_handle(os_file_handle fd)
{
    return xila_file_system_is_stdout(fd);
}

bool os_is_stderr_handle(os_file_handle fd)
{
    return xila_file_system_is_stderr(fd);
}
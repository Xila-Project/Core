#ifndef _XILA_H_INCLUDED
#define _XILA_H_INCLUDED

#include "platform_common.h"
#include "platform_wasi_types.h"
#include "platform_api_extension.h"

#define Rust_function __attribute__((weak))

// - Input/Output
int Rust_function Xila_print(const char *format);
int Rust_function Xila_virtual_print_formatted(const char *format, va_list ap);

// - CPU
void Rust_function Xila_flush_data_cache();
void Rust_function Xila_flush_instruction_cache(void *Start, size_t Length);
 
// - File system
typedef enum
{
    Failed_to_initialize_file_system = 1,
    Permission_denied,
    Not_found,
    Already_exists,
    Directory_already_exists,
    File_system_full,
    File_system_error,
    Invalid_path,
    Invalid_file,
    Invalid_directory,
    Invalid_symbolic_link,
    Unknown,
    Invalid_identifier,
    Failed_to_get_task_informations,
    Too_many_mounted_file_systems,
    Poisoned_lock,
    Too_many_open_files,
    Internal_error,
    Invalid_mode,
    Unsupported_operation,
    Ressource_busy,
    Already_initialized,
    Not_initialized,
    Failed_to_get_users_manager_instance,
    Failed_to_get_task_manager_instance,
    Invalid_input,
    Other,
} File_system_error_type;

typedef uint32_t File_system_result_type;

enum
{
    Xila_file_system_type_file,
    Xila_file_system_type_directory,
    Xila_file_system_type_block_device,
    Xila_file_system_type_character_device,
    Xila_file_system_type_pipe,
    Xila_file_system_type_socket,
    Xila_file_system_type_symbolic_link,
};

typedef uint8_t File_system_type_type;

typedef struct
{
    uint16_t File_system;
    uint64_t Inode;
    uint64_t Links;
    uint64_t Size;
    uint64_t Last_access;
    uint64_t Last_modification;
    uint64_t Last_status_change;
    File_system_type_type Type;
} Xila_file_system_statistics_type;

typedef uint8_t Xila_file_system_mode_type;

extern const Xila_file_system_mode_type Xila_file_system_mode_read_bit;
extern const Xila_file_system_mode_type Xila_file_system_mode_write_bit;

__wasi_errno_t
Into_Wasi_Error(File_system_result_type Error);
void Into_WASI_file_statistics(const Xila_file_system_statistics_type *Statistics, __wasi_filestat_t *WASI_Statistics);

File_system_result_type Rust_function Xila_get_file_statistics(os_file_handle Handle, Xila_file_system_statistics_type *Buffer);
File_system_result_type Rust_function Xila_get_file_statistics_at(os_file_handle Handle, const char *Path, struct __wasi_filestat_t *Buffer, __wasi_lookupflags_t Flags);
File_system_result_type Rust_function Xila_get_file_flags(os_file_handle Handle, __wasi_fdflags_t *Flags);
File_system_result_type Rust_function Xila_set_file_flags(os_file_handle Handle, __wasi_fdflags_t Flags);
File_system_result_type Rust_function Xila_synchronize_file_data(os_file_handle Handle, bool metadata);
File_system_result_type Rust_function Xila_pre_open_directory(const char *Path, os_file_handle *Handle);
File_system_result_type Rust_function Xila_open_at(os_file_handle Handle, const char *Path, __wasi_oflags_t O_Flags, __wasi_fdflags_t Fd_flags, __wasi_lookupflags_t Lookup_flags, wasi_libc_file_access_mode Access_mode, os_file_handle *New_handle);
File_system_result_type Rust_function Xila_file_system_get_access_mode(os_file_handle Handle, uint8_t *Access_mode);
File_system_result_type Rust_function Xila_file_system_close(os_file_handle Handle);
File_system_result_type Rust_function Xila_positioned_read_vectored(os_file_handle Handle, const struct __wasi_iovec_t *IOV, int IOV_len, __wasi_filesize_t Offset, size_t *NRead);
File_system_result_type Rust_function Xila_positioned_write_vectored(os_file_handle Handle, const struct __wasi_ciovec_t *IOV, int IOV_len, __wasi_filesize_t Offset, size_t *NWritten);
File_system_result_type Rust_function Xila_file_system_read_vectored(os_file_handle Handle, void *Buffer[], const size_t Lengths[], size_t IOV_len, size_t *NRead);
File_system_result_type Rust_function Xila_file_system_write_vectored(os_file_handle Handle, const void *Buffer[], const size_t Lengths[], size_t IOV_len, size_t *NWritten);
File_system_result_type Rust_function Xila_allocate_file(os_file_handle handle, __wasi_filesize_t offset, __wasi_filesize_t length);
File_system_result_type Rust_function Xila_truncate_file(os_file_handle handle, __wasi_filesize_t size);
File_system_result_type Rust_function Xila_set_file_times(os_file_handle handle, __wasi_timestamp_t ATime, __wasi_timestamp_t MTime, __wasi_fstflags_t Fst_flags);
File_system_result_type Rust_function Xila_set_file_times_at(os_file_handle handle, const char *Path, size_t Path_length, __wasi_timestamp_t ATime, __wasi_timestamp_t MTime, __wasi_fstflags_t Fst_flags);
File_system_result_type Rust_function Xila_read_link_at(os_file_handle handle, const char *Path, char *Buffer, size_t Buffer_size, size_t *Buffer_used);
File_system_result_type Rust_function Xila_create_link_at(os_file_handle Old_handle, const char *Old_path, os_file_handle New_handle, const char *New_path, bool Follow);
File_system_result_type Rust_function Xila_create_symbolic_link_at(const char *Target_path, os_file_handle New_handle, const char *Link_path);
File_system_result_type Rust_function Xila_create_directory(os_file_handle Handle, const char *Path);
File_system_result_type Rust_function Xila_create_directory_at(os_file_handle Handle, const char *Path);
File_system_result_type Rust_function Xila_rename_at(os_file_handle Old_handle, const char *Old_path, os_file_handle New_handle, const char *New_path);
File_system_result_type Rust_function Xila_unlink_at(os_file_handle Handle, const char *Path, bool Is_directory);
File_system_result_type Rust_function Xila_set_position(os_file_handle Handle, __wasi_filedelta_t Offset, __wasi_whence_t Whence, __wasi_filesize_t *New_offset);
File_system_result_type Rust_function Xila_get_advisory_information(os_file_handle Handle, __wasi_advice_t Advice, __wasi_filesize_t Offset, __wasi_filesize_t Len);
File_system_result_type Rust_function Xila_file_system_is_terminal(os_file_handle Handle);
File_system_result_type Rust_function Xila_open_directory(os_file_handle Handle, os_dir_stream *Dir_stream);
File_system_result_type Rust_function Xila_rewind_directory(os_dir_stream Dir_stream);
File_system_result_type Rust_function Xila_set_directory_position(os_dir_stream Dir_stream, uint64_t Position);
File_system_result_type Rust_function Xila_read_directory(os_dir_stream Dir_stream, __wasi_dirent_t *Buffer, const char **Buffer_used);
File_system_result_type Rust_function Xila_close_directory(os_dir_stream Dir_stream);
char *Rust_function Xila_resolve_path(const char *Path, char *Resolved_path);
bool Rust_function Xila_file_system_is_stdin(os_file_handle File);
bool Rust_function Xila_file_system_is_stdout(os_file_handle File);
bool Rust_function Xila_file_system_is_stderr(os_file_handle File);

// - Socket
typedef uint32_t Socket_return_type;
typedef bh_sockaddr_t Socket_address_type;

Socket_return_type Rust_function Xila_socket_create(bh_socket_t *Socket, bool Is_IPv4, bool Is_TCP);
Socket_return_type Rust_function Xila_socket_bind(bh_socket_t Socket, const char *Address, uint16_t *Port);
Socket_return_type Rust_function Xila_socket_set_timeout(bh_socket_t Socket, uint32_t Timeout);
Socket_return_type Rust_function Xila_socket_listen(bh_socket_t Socket, size_t Maximum_clients);
Socket_return_type Rust_function Xila_socket_accept(bh_socket_t Socket, bh_socket_t *New_socket, void *Address, size_t *Address_length);
Socket_return_type Rust_function Xila_socket_connect(bh_socket_t Socket, const char *Address, uint16_t Port);
Socket_return_type Rust_function Xila_socket_receive(bh_socket_t Socket, void *Buffer, size_t Length);
Socket_return_type Rust_function Xila_socket_receive_from(bh_socket_t Socket, void *Buffer, size_t Length, int Flags, Socket_address_type *Address);
Socket_return_type Rust_function Xila_socket_send(bh_socket_t Socket, const void *Buffer, size_t Length);
Socket_return_type Rust_function Xila_socket_send_to(bh_socket_t Socket, const void *Buffer, size_t Length, int Flags, const Socket_address_type *Address);
Socket_return_type Rust_function Xila_socket_close(bh_socket_t Socket);
Socket_return_type Rust_function Xila_socket_shutdown(bh_socket_t Socket);
Socket_return_type Rust_function Xila_socket_inet_network(bool Is_IPv4, const char *Address, bh_ip_addr_buffer_t *Out);
Socket_return_type Rust_function Xila_socket_address_resolve(const char *Host, const char *Service, uint8_t *Hint_is_TCP, uint8_t *Hint_is_IPv4, bh_addr_info_t *Address_informations, size_t Address_informations_length, size_t *Address_informations_count);
Socket_return_type Rust_function Xila_socket_address_local(bh_socket_t Socket, Socket_address_type *Out);
Socket_return_type Rust_function Xila_socket_address_remote(bh_socket_t Socket, Socket_address_type *Out);
Socket_return_type Rust_function Xila_socket_set_send_buffer_size(bh_socket_t Socket, size_t Size);
Socket_return_type Rust_function Xila_socket_get_send_buffer_size(bh_socket_t Socket, size_t *Size);
Socket_return_type Rust_function Xila_socket_set_receive_buffer_size(bh_socket_t Socket, size_t Size);
Socket_return_type Rust_function Xila_socket_get_receive_buffer_size(bh_socket_t Socket, size_t *Size);
Socket_return_type Rust_function Xila_socket_set_keep_alive(bh_socket_t Socket, bool Enable);
Socket_return_type Rust_function Xila_socket_get_keep_alive(bh_socket_t Socket, bool *Enable);
Socket_return_type Rust_function Xila_socket_set_send_timeout(bh_socket_t Socket, uint64_t Timeout);
Socket_return_type Rust_function Xila_socket_get_send_timeout(bh_socket_t Socket, uint64_t *Timeout);
Socket_return_type Rust_function Xila_socket_set_receive_timeout(bh_socket_t Socket, uint64_t Timeout);
Socket_return_type Rust_function Xila_socket_get_receive_timeout(bh_socket_t Socket, uint64_t *Timeout);
Socket_return_type Rust_function Xila_socket_set_reuse_address(bh_socket_t Socket, bool Enable);
Socket_return_type Rust_function Xila_socket_get_reuse_address(bh_socket_t Socket, bool *Enable);
Socket_return_type Rust_function Xila_socket_set_reuse_port(bh_socket_t Socket, bool Enable);
Socket_return_type Rust_function Xila_socket_get_reuse_port(bh_socket_t Socket, bool *Enable);
Socket_return_type Rust_function Xila_socket_set_linger(bh_socket_t Socket, bool Enable, uint64_t Time);
Socket_return_type Rust_function Xila_socket_get_linger(bh_socket_t Socket, bool *Enable, uint64_t *Time);
Socket_return_type Rust_function Xila_socket_set_tcp_no_delay(bh_socket_t Socket, bool Enable);
Socket_return_type Rust_function Xila_socket_get_tcp_no_delay(bh_socket_t Socket, bool *Enable);
Socket_return_type Rust_function Xila_socket_set_tcp_quick_ack(bh_socket_t Socket, bool Enable);
Socket_return_type Rust_function Xila_socket_get_tcp_quick_ack(bh_socket_t Socket, bool *Enable);
Socket_return_type Rust_function Xila_socket_set_tcp_keep_idle(bh_socket_t Socket, uint32_t Time);
Socket_return_type Rust_function Xila_socket_get_tcp_keep_idle(bh_socket_t Socket, uint32_t *Time);
Socket_return_type Rust_function Xila_socket_set_tcp_keep_interval(bh_socket_t Socket, uint32_t Time);
Socket_return_type Rust_function Xila_socket_get_tcp_keep_interval(bh_socket_t Socket, uint32_t *Time);
Socket_return_type Rust_function Xila_socket_set_tcp_fast_open_connect(bh_socket_t Socket, bool Enable);
Socket_return_type Rust_function Xila_socket_get_tcp_fast_open_connect(bh_socket_t Socket, bool *Enable);
Socket_return_type Rust_function Xila_socket_set_ip_multicast_loop(bh_socket_t Socket, bool Is_IPv6, bool Enable);
Socket_return_type Rust_function Xila_socket_get_ip_multicast_loop(bh_socket_t Socket, bool Is_IPv6, bool *Enable);
Socket_return_type Rust_function Xila_socket_set_ip_address_membership(bh_socket_t Socket, bh_ip_addr_buffer_t *Group_multicast_address, uint32_t Interface_address, bool Is_IPv6);
Socket_return_type Rust_function Xila_socket_drop_ip_address_membership(bh_socket_t Socket, bh_ip_addr_buffer_t *Group_multicast_address, uint32_t Interface_address, bool Is_IPv6);
Socket_return_type Rust_function Xila_socket_set_ip_time_to_live(bh_socket_t Socket, uint8_t TTL);
Socket_return_type Rust_function Xila_socket_get_ip_time_to_live(bh_socket_t Socket, uint8_t *TTL);
Socket_return_type Rust_function Xila_socket_set_ip_multicast_time_to_live(bh_socket_t Socket, uint8_t TTL);
Socket_return_type Rust_function Xila_socket_get_ip_multicast_time_to_live(bh_socket_t Socket, uint8_t *TTL);
Socket_return_type Rust_function Xila_socket_set_ipv6_only(bh_socket_t Socket, bool Enable);
Socket_return_type Rust_function Xila_socket_get_ipv6_only(bh_socket_t Socket, bool *Enable);
Socket_return_type Rust_function Xila_socket_set_broadcast(bh_socket_t Socket, bool Enable);
Socket_return_type Rust_function Xila_socket_get_broadcast(bh_socket_t Socket, bool *Enable);

// - Memory
// - - Regular allocation
void *Rust_function Xila_memory_allocate(size_t Size);
void *Rust_function Xila_memory_reallocate(void *Pointer, size_t Size);
void Rust_function Xila_memory_deallocate(void *Pointer);

// - - Mappping
typedef uint8_t Xila_memory_protection_type;
typedef uint8_t Xila_memory_flag_type;

extern const Xila_memory_protection_type Xila_memory_protection_read;
extern const Xila_memory_protection_type Xila_memory_protection_write;
extern const Xila_memory_protection_type Xila_memory_protection_execute;

extern const Xila_memory_flag_type Xila_memory_flag_anonymous;
extern const Xila_memory_flag_type Xila_memory_flag_private;
extern const Xila_memory_flag_type Xila_memory_flag_fixed;

void *Rust_function Xila_memory_allocate_custom(void *Pointer, size_t Size, uint8_t Alignment, Xila_memory_protection_type Protection, Xila_memory_flag_type Flags);
void Rust_function Xila_memory_deallocate_custom(void *Pointer, size_t Size);
// void *Rust_function Xila_remap_memory(void *Old_pointer, size_t Old_size, size_t New_size);
int Rust_function Xila_memory_protect(void *Pointer, size_t Size, int Protection);
size_t Rust_function Xila_memory_get_page_size();

// - Time
uint64_t Rust_function Xila_get_boot_time_microseconds();
uint64_t Rust_function Xila_get_cpu_time_microseconds();

// - Clock
uint64_t Rust_function Xila_get_clock_resolution(uint32_t Clock_identifier);
uint64_t Rust_function Xila_get_clock_time(uint32_t Clock_identifier, uint64_t Precision);

// - Thread
// - - Management
int Rust_function Xila_thread_create(korp_tid *Thread, thread_start_routine_t Start, void *arg, unsigned int stack_size);
int Rust_function Xila_thread_create_with_priority(korp_tid *p_tid, thread_start_routine_t start, void *arg, unsigned int stack_size, int priority);
int Rust_function Xila_thread_join(korp_tid Thread, void **Return_value);
int Rust_function Xila_thread_detach(korp_tid Thread);
void Rust_function Xila_thread_exit(void *Return_value);
int Rust_function Xila_sleep_microsecond(uint32_t Microseconds);

korp_tid Rust_function Xila_get_current_thread_identifier();
uint8_t *Rust_function Xila_get_thread_stack_boundary();

int Rust_function Xila_dumps_memory_informations(char *Buffer, size_t Buffer_size);

// - - Mutex
bool Rust_function Xila_initialize_recursive_mutex(korp_mutex *mutex);
bool Rust_function Xila_initialize_mutex(korp_mutex *mutex);
bool Rust_function Xila_destroy_mutex(korp_mutex *mutex);
bool Rust_function Xila_lock_mutex(korp_mutex *mutex);
bool Rust_function Xila_unlock_mutex(korp_mutex *mutex);

// - - Condition variable
int Rust_function Xila_initialize_condition_variable(korp_cond *cond);
int Rust_function Xila_destroy_condition_variable(korp_cond *cond);
int Rust_function Xila_wait_condition_variable(korp_cond *cond, korp_mutex *mutex);
int Rust_function Xila_wait_condition_variable_with_timeout(korp_cond *cond, korp_mutex *mutex, uint64_t timeout);
int Rust_function Xila_signal_condition_variable(korp_cond *cond);
int Rust_function Xila_broadcast_condition_variable(korp_cond *cond);

// - - Read/Write lock
int Rust_function Xila_initialize_rwlock(korp_rwlock *rwlock);
int Rust_function Xila_destroy_rwlock(korp_rwlock *rwlock);
int Rust_function Xila_read_rwlock(korp_rwlock *rwlock);
int Rust_function Xila_write_rwlock(korp_rwlock *rwlock);
int Rust_function Xila_unlock_rwlock(korp_rwlock *rwlock);

// - - Semaphore
korp_sem *Rust_function Xila_open_semaphore(const char *Name, int Open_flag, int Mode, int Value);
int Rust_function Xila_close_semaphore(korp_sem *Sem);
int Rust_function Xila_wait_semaphore(korp_sem *Sem);
int Rust_function Xila_try_wait_semaphore(korp_sem *Sem);
int Rust_function Xila_post_semaphore(korp_sem *Sem);
int Rust_function Xila_get_semaphore_value(korp_sem *Sem);
int Rust_function Xila_unlink_semaphore(const char *Name);

// - - Blocking operation
int Rust_function Xila_initialize_blocking_operation();
void Rust_function Xila_begin_blocking_operation();
void Rust_function Xila_end_blocking_operation();
int Rust_function Xila_wakeup_blocking_operation(korp_tid Thread);

#endif
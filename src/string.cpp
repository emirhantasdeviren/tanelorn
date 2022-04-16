#include <windows.h>

#include "./defines.hpp"
#include "./string.hpp"

namespace tanelorn {
    String::String() : data(nullptr), len(0), capacity(0) {}
    String::String(const char *s) {
        if (s == nullptr) {
            this->data = nullptr;
            this->len = 0;
            this->capacity = 0;
        } else {
            u64 slen = 0;
            while (*(s + slen++) != '\0') {}
            this->data = static_cast<u8 *>(
                VirtualAlloc(nullptr, slen - 1, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE));
            this->len = slen - 1;
            this->capacity = slen - 1;

            CopyMemory(this->data, s, slen - 1);
        }
    }

    String::~String() {
        if (data != nullptr) {
            VirtualFree(data, 0, MEM_RELEASE);
        }
    }

    u8 *String::as_ptr() const noexcept { return this->data; }

    // TODO: add terminating null-character
    char *String::as_c_str() { return reinterpret_cast<char *>(this->data); }

    void String::push(u8 ch) {
        if (this->data == nullptr) {
            this->data = static_cast<u8 *>(
                VirtualAlloc(nullptr, 1, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE));
            this->capacity = 1;
            *(this->data) = ch;
            this->len = 1;
        } else if (this->len == this->capacity) {
            u8 *next_addr =
                static_cast<u8 *>(VirtualAlloc(this->data + this->capacity, this->capacity,
                                               MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE));

            if (next_addr == nullptr) {
                u8 *new_addr = static_cast<u8 *>(VirtualAlloc(
                    nullptr, this->capacity * 2, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE));

                memcpy(new_addr, this->data, this->len);
                VirtualFree(this->data, 0, MEM_RELEASE);
                this->data = new_addr;
            }

            *(this->data + this->len) = ch;
            this->len++;
            this->capacity *= 2;
        } else {
            *(this->data + this->len) = ch;
            this->len++;
        }
    }
} // namespace tanelorn

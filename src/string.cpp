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
                VirtualAlloc(nullptr, slen, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE));
            this->len = slen;
            this->capacity = slen;

            CopyMemory(this->data, s, slen);
        }
    }

    String::~String() {
        if (data != nullptr) {
            VirtualFree(data, 0, MEM_RELEASE);
        }
    }

    u8 *String::as_ptr() { return this->data; }

    // TODO: add terminating null-character
    char *String::as_c_str() { return reinterpret_cast<char *>(this->data); }
} // namespace tanelorn

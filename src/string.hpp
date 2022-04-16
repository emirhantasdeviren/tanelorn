#include "./defines.hpp"

namespace tanelorn {
    class String {
      private:
        u8 *data;
        u64 len;
        u64 capacity;

      public:
        String(const char *s);
        String();
        ~String();
        String(const String &) = delete;

        u8 *as_ptr() const noexcept;
        char *as_c_str();
        void push(u8 ch);
    };
}; // namespace tanelorn
pub(crate) const RUNTIME_C: &str = r#"
typedef unsigned long long size_t;
typedef long long int64_t;

int printf(const char* format, ...);
int sprintf(char* str, const char* format, ...);
void* malloc(size_t size);
void* memset(void* ptr, int value, size_t num);
void free(void* ptr);
size_t strlen(const char* str);
char* strcpy(char* destination, const char* source);
char* strcat(char* destination, const char* source);
char* strstr(const char* haystack, const char* needle);
int strcmp(const char* s1, const char* s2);
int strncmp(const char* s1, const char* s2, size_t n);
void* memcpy(void* dest, const void* src, size_t n);
char* strtok(char* str, const char* delimiters);
long long strtoll(const char* str, char** endptr, int base);
double strtod(const char* str, char** endptr);

int global_argc = 0;
char** global_argv = 0;

void* n0_c_alloc(size_t size) {
    void* ptr = malloc(size);
    if (ptr) memset(ptr, 0, size);
    return ptr;
}

void n0_c_store_int(void* addr, size_t offset, size_t val) {
    *(size_t*)((char*)addr + offset) = val;
}

void n0_c_store_string(void* addr, size_t offset, char* val) {
    *(char**)((char*)addr + offset) = val;
}

size_t n0_c_load_int(void* addr, size_t offset) {
    return *(size_t*)((char*)addr + offset);
}

char* n0_c_load_string(void* addr, size_t offset) {
    return *(char**)((char*)addr + offset);
}

char* n0_c_interpolate(char* s1, char* s2) {
    char* res = malloc(strlen(s1) + strlen(s2) + 1);
    if (res) {
        strcpy(res, s1);
        strcat(res, s2);
    }
    return res;
}

char* n0_int_to_string(int64_t n) {
    char buf[64];
    int idx = 0;
    int is_neg = 0;
    unsigned long long val;
    if (n < 0) {
        is_neg = 1;
        val = -n;
    } else {
        val = n;
    }
    do {
        buf[idx++] = (val % 10) + '0';
        val /= 10;
    } while (val > 0);
    if (is_neg) {
        buf[idx++] = '-';
    }
    char* res = malloc(idx + 1);
    if (res) {
        for (int j = 0; j < idx; j++) {
            res[j] = buf[idx - 1 - j];
        }
        res[idx] = '\0';
    }
    return res;
}

char* n0_float_to_string(double f) {
    char buf[128];
    int idx = 0;
    if (f < 0) {
        buf[idx++] = '-';
        f = -f;
    }
    f += 0.0000005;
    unsigned long long ipart = (unsigned long long)f;
    double fpart = f - (double)ipart;
    char temp[64];
    int temp_idx = 0;
    do {
        temp[temp_idx++] = (ipart % 10) + '0';
        ipart /= 10;
    } while (ipart > 0);
    for (int j = 0; j < temp_idx; j++) {
        buf[idx++] = temp[temp_idx - 1 - j];
    }
    buf[idx++] = '.';
    for (int j = 0; j < 6; j++) {
        fpart *= 10;
        int digit = (int)fpart;
        buf[idx++] = digit + '0';
        fpart -= digit;
    }
    buf[idx] = '\0';
    char* res = malloc(idx + 1);
    if (res) strcpy(res, buf);
    return res;
}

char* n0_bool_to_string(int b) {
    const char* s = b ? "true" : "false";
    char* res = malloc(strlen(s) + 1);
    if (res) strcpy(res, s);
    return res;
}

char* n0_string_concat(char** parts, int count) {
    size_t total_len = 0;
    for (int i = 0; i < count; i++) {
        if (parts[i]) {
            total_len += strlen(parts[i]);
        }
    }
    char* res = malloc(total_len + 1);
    if (res) {
        res[0] = '\0';
        for (int i = 0; i < count; i++) {
            if (parts[i]) {
                strcat(res, parts[i]);
            }
        }
    }
    return res;
}

size_t n0_c_argc() {
    return (size_t)global_argc;
}

char* n0_c_argv(size_t index) {
    if (global_argv && index < global_argc) {
        return global_argv[index];
    }
    return "";
}

void n0_show_string(const char* s) { printf("%s\n", s); }
void n0_show_int(size_t i) { printf("%llu\n", (unsigned long long)i); }
void n0_show_float(double f) { printf("%f\n", f); }

// Option Helpers
void* n0_make_some(int64_t val) {
    void* opt = malloc(32);
    if (opt) {
        *(int64_t*)((char*)opt + 8) = 1;
        *(int64_t*)((char*)opt + 16) = 0;
        *(int64_t*)((char*)opt + 24) = val;
    }
    return opt;
}

void* n0_make_none() {
    void* opt = malloc(32);
    if (opt) {
        *(int64_t*)((char*)opt + 8) = 0;
        *(int64_t*)((char*)opt + 16) = 1;
        *(int64_t*)((char*)opt + 24) = 0;
    }
    return opt;
}

// String Methods
int64_t n0_str_len(char* s) {
    return s ? (int64_t)strlen(s) : 0;
}

int64_t n0_str_contains(char* s, char* x) {
    if (!s || !x) return 0;
    return strstr(s, x) != 0;
}

int64_t n0_str_starts_with(char* s, char* x) {
    if (!s || !x) return 0;
    size_t len_s = strlen(s);
    size_t len_x = strlen(x);
    if (len_x > len_s) return 0;
    return strncmp(s, x, len_x) == 0;
}

int64_t n0_str_ends_with(char* s, char* x) {
    if (!s || !x) return 0;
    size_t len_s = strlen(s);
    size_t len_x = strlen(x);
    if (len_x > len_s) return 0;
    return strcmp(s + len_s - len_x, x) == 0;
}

char* n0_str_upper(char* s) {
    if (!s) return "";
    size_t len = strlen(s);
    char* res = malloc(len + 1);
    if (res) {
        for (size_t i = 0; i < len; i++) {
            char c = s[i];
            if (c >= 'a' && c <= 'z') c = c - 'a' + 'A';
            res[i] = c;
        }
        res[len] = '\0';
    }
    return res;
}

char* n0_str_lower(char* s) {
    if (!s) return "";
    size_t len = strlen(s);
    char* res = malloc(len + 1);
    if (res) {
        for (size_t i = 0; i < len; i++) {
            char c = s[i];
            if (c >= 'A' && c <= 'Z') c = c - 'A' + 'a';
            res[i] = c;
        }
        res[len] = '\0';
    }
    return res;
}

char* n0_str_trim(char* s) {
    if (!s) return "";
    size_t start = 0;
    size_t end = strlen(s);
    while (start < end && (s[start] == ' ' || s[start] == '\t' || s[start] == '\r' || s[start] == '\n')) {
        start++;
    }
    while (end > start && (s[end - 1] == ' ' || s[end - 1] == '\t' || s[end - 1] == '\r' || s[end - 1] == '\n')) {
        end--;
    }
    size_t len = end - start;
    char* res = malloc(len + 1);
    if (res) {
        for (size_t i = 0; i < len; i++) {
            res[i] = s[start + i];
        }
        res[len] = '\0';
    }
    return res;
}

void* n0_str_split(char* s, char* delim) {
    void* list = malloc(24);
    if (!list) return 0;
    memset(list, 0, 24);
    if (!s || !delim || strlen(s) == 0) {
        return list;
    }
    int count = 0;
    char* s_copy = malloc(strlen(s) + 1);
    strcpy(s_copy, s);
    char* token = strtok(s_copy, delim);
    while (token) {
        count++;
        token = strtok(0, delim);
    }
    free(s_copy);
    if (count > 0) {
        char** data = malloc(count * 8);
        memset(data, 0, count * 8);
        s_copy = malloc(strlen(s) + 1);
        strcpy(s_copy, s);
        int idx = 0;
        token = strtok(s_copy, delim);
        while (token) {
            char* t = malloc(strlen(token) + 1);
            strcpy(t, token);
            data[idx++] = t;
            token = strtok(0, delim);
        }
        free(s_copy);
        *(int64_t*)((char*)list + 0) = count;
        *(char***)((char*)list + 8) = data;
        *(int64_t*)((char*)list + 16) = count;
    }
    return list;
}

char* n0_str_replace(char* s, char* from, char* to) {
    if (!s || !from || !to) return "";
    size_t len_s = strlen(s);
    size_t len_from = strlen(from);
    size_t len_to = strlen(to);
    if (len_from == 0) return s;
    int count = 0;
    char* p = s;
    while ((p = strstr(p, from)) != 0) {
        count++;
        p += len_from;
    }
    size_t new_len = len_s + count * (len_to - len_from);
    char* res = malloc(new_len + 1);
    if (res) {
        char* dst = res;
        char* src = s;
        while ((p = strstr(src, from)) != 0) {
            size_t chunk = p - src;
            memcpy(dst, src, chunk);
            dst += chunk;
            memcpy(dst, to, len_to);
            dst += len_to;
            src = p + len_from;
        }
        strcpy(dst, src);
    }
    return res;
}

char* n0_str_slice(char* s, int64_t start, int64_t end) {
    if (!s) return "";
    int64_t len = (int64_t)strlen(s);
    if (start < 0) start = 0;
    if (end > len) end = len;
    if (start > end) return "";
    int64_t slice_len = end - start;
    char* res = malloc(slice_len + 1);
    if (res) {
        memcpy(res, s + start, slice_len);
        res[slice_len] = '\0';
    }
    return res;
}

void* n0_str_to_int(char* s) {
    if (!s) return n0_make_none();
    while (*s == ' ' || *s == '\t' || *s == '\r' || *s == '\n') s++;
    if (*s == '\0') return n0_make_none();
    char* endptr;
    long long val = strtoll(s, &endptr, 10);
    if (endptr == s) {
        return n0_make_none();
    }
    return n0_make_some(val);
}

void* n0_str_to_float(char* s) {
    if (!s) return n0_make_none();
    while (*s == ' ' || *s == '\t' || *s == '\r' || *s == '\n') s++;
    if (*s == '\0') return n0_make_none();
    char* endptr;
    double val = strtod(s, &endptr);
    if (endptr == s) {
        return n0_make_none();
    }
    union {
        double f;
        int64_t i;
    } u;
    u.f = val;
    return n0_make_some(u.i);
}

// List Methods
int64_t n0_list_len(void* list) {
    if (!list) return 0;
    return *(int64_t*)((char*)list + 16);
}

void n0_list_push(void* list, int64_t val) {
    if (!list) return;
    int64_t cap = *(int64_t*)((char*)list + 0);
    int64_t* data = *(int64_t**)((char*)list + 8);
    int64_t len = *(int64_t*)((char*)list + 16);
    if (len >= cap) {
        int64_t new_cap = cap == 0 ? 4 : cap * 2;
        int64_t* new_data = malloc(new_cap * 8);
        if (new_data) {
            memset(new_data, 0, new_cap * 8);
            if (data) {
                for (int i = 0; i < len; i++) {
                    new_data[i] = data[i];
                }
                free(data);
            }
            *(int64_t**)((char*)list + 8) = new_data;
            *(int64_t*)((char*)list + 0) = new_cap;
            data = new_data;
        }
    }
    if (data) {
        data[len] = val;
        *(int64_t*)((char*)list + 16) = len + 1;
    }
}

void* n0_list_pop(void* list) {
    if (!list) return n0_make_none();
    int64_t len = *(int64_t*)((char*)list + 16);
    if (len <= 0) return n0_make_none();
    int64_t* data = *(int64_t**)((char*)list + 8);
    if (!data) return n0_make_none();
    int64_t val = data[len - 1];
    *(int64_t*)((char*)list + 16) = len - 1;
    return n0_make_some(val);
}

int64_t n0_list_contains_int(void* list, int64_t val) {
    if (!list) return 0;
    int64_t len = *(int64_t*)((char*)list + 16);
    int64_t* data = *(int64_t**)((char*)list + 8);
    if (!data) return 0;
    for (int64_t i = 0; i < len; i++) {
        if (data[i] == val) return 1;
    }
    return 0;
}

int64_t n0_list_contains_str(void* list, char* val) {
    if (!list || !val) return 0;
    int64_t len = *(int64_t*)((char*)list + 16);
    char** data = *(char***)((char*)list + 8);
    if (!data) return 0;
    for (int64_t i = 0; i < len; i++) {
        if (data[i] && strcmp(data[i], val) == 0) return 1;
    }
    return 0;
}

void* n0_list_first(void* list) {
    if (!list) return n0_make_none();
    int64_t len = *(int64_t*)((char*)list + 16);
    if (len <= 0) return n0_make_none();
    int64_t* data = *(int64_t**)((char*)list + 8);
    if (!data) return n0_make_none();
    return n0_make_some(data[0]);
}

void* n0_list_last(void* list) {
    if (!list) return n0_make_none();
    int64_t len = *(int64_t*)((char*)list + 16);
    if (len <= 0) return n0_make_none();
    int64_t* data = *(int64_t**)((char*)list + 8);
    if (!data) return n0_make_none();
    return n0_make_some(data[len - 1]);
}

// Map Methods
void* n0_map_get(void* map, char* key) {
    if (!map || !key) return n0_make_none();
    int64_t n = *(int64_t*)((char*)map + 24);
    char** keys = *(char***)((char*)map + 8);
    int64_t* vals = *(int64_t**)((char*)map + 16);
    if (!keys || !vals) return n0_make_none();
    for (int64_t i = 0; i < n; i++) {
        if (keys[i] && strcmp(keys[i], key) == 0) {
            return n0_make_some(vals[i]);
        }
    }
    return n0_make_none();
}

void n0_map_set(void* map, char* key, int64_t val) {
    if (!map || !key) return;
    int64_t cap = *(int64_t*)((char*)map + 0);
    char** keys = *(char***)((char*)map + 8);
    int64_t* vals = *(int64_t**)((char*)map + 16);
    int64_t n = *(int64_t*)((char*)map + 24);
    for (int64_t i = 0; i < n; i++) {
        if (keys[i] && strcmp(keys[i], key) == 0) {
            vals[i] = val;
            return;
        }
    }
    if (n >= cap) {
        int64_t new_cap = cap == 0 ? 4 : cap * 2;
        char** new_keys = malloc(new_cap * 8);
        int64_t* new_vals = malloc(new_cap * 8);
        if (new_keys && new_vals) {
            memset(new_keys, 0, new_cap * 8);
            memset(new_vals, 0, new_cap * 8);
            if (keys && vals) {
                for (int64_t i = 0; i < n; i++) {
                    new_keys[i] = keys[i];
                    new_vals[i] = vals[i];
                }
                free(keys);
                free(vals);
            }
            *(char***)((char*)map + 8) = new_keys;
            *(int64_t**)((char*)map + 16) = new_vals;
            *(int64_t*)((char*)map + 0) = new_cap;
            keys = new_keys;
            vals = new_vals;
        }
    }
    if (keys && vals) {
        char* key_copy = malloc(strlen(key) + 1);
        strcpy(key_copy, key);
        keys[n] = key_copy;
        vals[n] = val;
        *(int64_t*)((char*)map + 24) = n + 1;
    }
}

int64_t n0_map_has(void* map, char* key) {
    if (!map || !key) return 0;
    int64_t n = *(int64_t*)((char*)map + 24);
    char** keys = *(char***)((char*)map + 8);
    if (!keys) return 0;
    for (int64_t i = 0; i < n; i++) {
        if (keys[i] && strcmp(keys[i], key) == 0) return 1;
    }
    return 0;
}

void* n0_map_keys(void* map) {
    void* list = malloc(24);
    if (!list) return 0;
    memset(list, 0, 24);
    if (!map) return list;
    int64_t n = *(int64_t*)((char*)map + 24);
    char** keys = *(char***)((char*)map + 8);
    if (n > 0 && keys) {
        char** new_data = malloc(n * 8);
        if (new_data) {
            for (int64_t i = 0; i < n; i++) {
                if (keys[i]) {
                    char* key_copy = malloc(strlen(keys[i]) + 1);
                    strcpy(key_copy, keys[i]);
                    new_data[i] = key_copy;
                } else {
                    new_data[i] = 0;
                }
            }
            *(int64_t*)((char*)list + 0) = n;
            *(char***)((char*)list + 8) = new_data;
            *(int64_t*)((char*)list + 16) = n;
        }
    }
    return list;
}

void* n0_map_values(void* map) {
    void* list = malloc(24);
    if (!list) return 0;
    memset(list, 0, 24);
    if (!map) return list;
    int64_t n = *(int64_t*)((char*)map + 24);
    int64_t* vals = *(int64_t**)((char*)map + 16);
    if (n > 0 && vals) {
        int64_t* new_data = malloc(n * 8);
        if (new_data) {
            for (int64_t i = 0; i < n; i++) {
                new_data[i] = vals[i];
            }
            *(int64_t*)((char*)list + 0) = n;
            *(int64_t**)((char*)list + 8) = new_data;
            *(int64_t*)((char*)list + 16) = n;
        }
    }
    return list;
}

void n0_map_delete(void* map, char* key) {
    if (!map || !key) return;
    int64_t n = *(int64_t*)((char*)map + 24);
    char** keys = *(char***)((char*)map + 8);
    int64_t* vals = *(int64_t**)((char*)map + 16);
    if (!keys || !vals) return;
    for (int64_t i = 0; i < n; i++) {
        if (keys[i] && strcmp(keys[i], key) == 0) {
            // Do not free keys[i] as it might be a static string constant.
            for (int64_t j = i; j < n - 1; j++) {
                keys[j] = keys[j + 1];
                vals[j] = vals[j + 1];
            }
            keys[n - 1] = 0;
            vals[n - 1] = 0;
            *(int64_t*)((char*)map + 24) = n - 1;
            return;
        }
    }
}

// Int/Float Methods
double n0_int_to_float(int64_t n) {
    return (double)n;
}

int64_t n0_float_to_int(double f) {
    return (int64_t)f;
}
"#;

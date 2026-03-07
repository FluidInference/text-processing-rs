#ifndef NEMO_TEXT_PROCESSING_H
#define NEMO_TEXT_PROCESSING_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

char* nemo_normalize(const char* input);
char* nemo_normalize_sentence(const char* input);
char* nemo_normalize_sentence_with_max_span(const char* input, uint32_t max_span_tokens);
void nemo_add_rule(const char* spoken, const char* written);
int32_t nemo_remove_rule(const char* spoken);
void nemo_clear_rules(void);
uint32_t nemo_rule_count(void);
void nemo_free_string(char* s);
const char* nemo_version(void);

/* Text Normalization (written → spoken) */
char* nemo_tn_normalize(const char* input);
char* nemo_tn_normalize_sentence(const char* input);
char* nemo_tn_normalize_sentence_with_max_span(const char* input, uint32_t max_span_tokens);

#ifdef __cplusplus
}
#endif

#endif /* NEMO_TEXT_PROCESSING_H */

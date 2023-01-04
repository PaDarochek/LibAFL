#include "common.h"
#include <stddef.h>

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-parameter"
EXT_FUNC(LLVMFuzzerInitialize, int, (int *argc, char ***argv), false);
EXT_FUNC(LLVMFuzzerCustomMutator, size_t,
         (uint8_t * Data, size_t Size, size_t MaxSize, unsigned int Seed),
         false);
EXT_FUNC(LLVMFuzzerCustomCrossOver, size_t,
         (const uint8_t *Data1, size_t Size1, const uint8_t *Data2,
          size_t Size2, uint8_t *Out, size_t MaxOutSize, unsigned int Seed),
         false);
EXT_FUNC_IMPL(LLVMFuzzerTestOneInput, int, (const uint8_t *Data, size_t Size),
              false) {
  return 0;
}

EXT_FUNC_IMPL(libafl_main, void, (void), false) {
}

EXT_FUNC(libafl_main, void, (void), false);
extern int LLVMFuzzerRunDriver(int *argc, char ***argv,
                               int (*UserCb)(const uint8_t *Data, size_t Size));

#ifndef FUZZER_NO_LINK_MAIN
EXT_FUNC_IMPL(main, int, (int argc, char **argv), false) {
  if (CHECK_WEAK_FN(libafl_main)) {
    libafl_main();
    return 0;
  }
  return LLVMFuzzerRunDriver(&argc, &argv, &LLVMFuzzerTestOneInput);
}

  #if defined(_WIN32)
// If we do not add the main, the MSVC linker fails with:
// LINK : fatal error LNK1561: entry point must be defined
int main(int argc, char **argv) {
  if (CHECK_WEAK_FN(libafl_main)) {
    libafl_main();
    return 0;
  }
  return LLVMFuzzerRunDriver(&argc, &argv, &LLVMFuzzerTestOneInput);
}
  #endif
#endif

#pragma GCC diagnostic pop

// take a page out of libfuzzer's book: static define __sancov_lowest_stack
// since we don't support it yet
// TODO support it
MAYBE_THREAD_LOCAL uintptr_t __sancov_lowest_stack;

EXPORT_FN int libafl_targets_has_libfuzzer_init() {
  return CHECK_WEAK_FN(LLVMFuzzerInitialize);
}

EXPORT_FN int libafl_targets_libfuzzer_init(int *argc, char ***argv) {
  if (libafl_targets_has_libfuzzer_init()) {
    return LLVMFuzzerInitialize(argc, argv);
  } else {
    return 0;
  }
}

EXPORT_FN int libafl_targets_has_libfuzzer_custom_mutator() {
  return CHECK_WEAK_FN(LLVMFuzzerCustomMutator);
}

// trust the user to check this appropriately :)
EXPORT_FN size_t libafl_targets_libfuzzer_custom_mutator(uint8_t     *Data,
                                                         size_t       Size,
                                                         size_t       MaxSize,
                                                         unsigned int Seed) {
  return LLVMFuzzerCustomMutator(Data, Size, MaxSize, Seed);
}

EXPORT_FN int libafl_targets_has_libfuzzer_custom_crossover() {
  return CHECK_WEAK_FN(LLVMFuzzerCustomCrossOver);
}

// trust the user to check this appropriately :)
EXPORT_FN size_t libafl_targets_libfuzzer_custom_crossover(
    const uint8_t *Data1, size_t Size1, const uint8_t *Data2, size_t Size2,
    uint8_t *Out, size_t MaxOutSize, unsigned int Seed) {
  return LLVMFuzzerCustomCrossOver(Data1, Size1, Data2, Size2, Out, MaxOutSize,
                                   Seed);
}

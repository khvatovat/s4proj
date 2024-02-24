// Filename: fingerprint_capture.h

#ifndef FINGERPRINT_CAPTURE_H
#define FINGERPRINT_CAPTURE_H

#include <Windows.h>
#include "lib/headers.h"
#include "lib/winbio_err.h"
#include "lib/winbio_types.h"
#include <WinBio.h>

#ifdef __cplusplus
extern "C" {
#endif

// Function declarations
void BmpSetImageData(void* bmp, const unsigned char* data, unsigned int width, unsigned int height);
void BmpSave(const void* bmp, const char* filename);
long CaptureSample();

#ifdef __cplusplus
}
#endif

#endif // FINGERPRINT_CAPTURE_H
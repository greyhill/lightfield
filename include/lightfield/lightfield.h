#pragma once

#include <stdint.h>
#include <math.h>
#include <stdlib.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <assert.h>

#include "CL/cl.h"

// Error checking
#include "lightfield_error.h"

// Thin lens
#include "lightfield_lens.h"

// Affine optics
#include "lightfield_optics.h"

// Angular plane
#include "lightfield_angular_plane.h"

// Plane geometry
#include "lightfield_plane_geometry.h"

// Light field pixels
#include "lightfield_lixel.h"

// Geometric vectors
#include "lightfield_vec3.h"

// Planes in space
#include "lightfield_flat.h"

// OpenCL utilities
#include "lightfield_cl.h"

// Light transport
#include "lightfield_transport.h"

// Simple camera
#include "lightfield_camera.h"


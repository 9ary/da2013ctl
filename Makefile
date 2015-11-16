NAME = da2013ctl

DEBUG = FALSE

CC = gcc

CFLAGS_COMMON = -Wall -Wextra \
		$(addprefix -I,$(INCLUDE) $(INCLUDE_EXT)) \
		-D'PROJ_NAME="$(NAME)"' \
		-MMD -MP \
		-pipe

ifeq ($(DEBUG),FALSE)
	CFLAGS_COMMON += -O2 -flto
else
	CFLAGS_COMMON += -O0 -g
endif

CFLAGS = $(CFLAGS_COMMON) -std=gnu11

LDFLAGS = $(CFLAGS_COMMON) \
	  -ludev

SRCS_C :=
OBJS = $(BUILT_SRCS_C:%.c=%.o) \
	$(addprefix $(OUT)/,$(SRCS_C:%.c=%.o))
INCLUDE :=
INCLUDE_EXT :=
BUILT_SRCS :=
BUILT_SRCS_C :=

OUT = out
EXE = $(OUT)/$(NAME)

PREFIX = "/usr/local"

CLEAN_SPEC :=

# Figure out where we are
define whereami
$(strip \
	$(eval LOCAL_MODULE_MAKEFILE := $$(lastword $$(MAKEFILE_LIST))) \
	$(if $(filter $(BUILD_SYSTEM)/% $(OUT_DIR)/%,$(LOCAL_MODULE_MAKEFILE)), \
		$(error my-dir must be called before including any other makefile.) \
	, \
		$(patsubst %/,%,$(dir $(LOCAL_MODULE_MAKEFILE))) \
	) \
)
endef

include rules.mk

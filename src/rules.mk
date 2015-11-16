src_LOCAL_PATH := $(call whereami)

SRCS_C += $(wildcard $(src_LOCAL_PATH)/*.c)
INCLUDE += $(src_LOCAL_PATH)

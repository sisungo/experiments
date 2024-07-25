/* SPDX-License-Identifier: GPL-2.0 WITH Linux-syscall-note */
/*
 * TrustedCell - User space API
 *
 * Copyright (C) 2024 sisungo <sisungo@icloud.com>
 */

#ifndef _UAPI_LINUX_TRUSTEDCELL_H
#define _UAPI_LINUX_TRUSTEDCELL_H

#define TRUSTEDCELL_CELL_IDENTIFIER_MAX 127
#define TRUSTEDCELL_CATEGORY_MAX 47
#define TRUSTEDCELL_ACTION_IDENTIFIER_MAX 31

#define TRUSTEDCELL_PROC_CATEGORY "~proc"

#define TRUSTEDCELL_XATTR_CATEGORY "security.tc_category"
#define TRUSTEDCELL_XATTR_OWNER "security.tc_owner"
#define TRUSTEDCELL_XATTR_CATEGORY_SUFFIX "tc_category"
#define TRUSTEDCELL_XATTR_OWNER_SUFFIX "tc_owner"

#endif /* _UAPI_LINUX_TRUSTEDCELL_H */

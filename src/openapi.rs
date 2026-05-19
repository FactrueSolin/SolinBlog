//! OpenAPI 文档定义
//!
//! 提供图床 API 的 OpenAPI 3.1 规范

use serde_json::json;

/// 生成 OpenAPI 文档 JSON
pub fn build_openapi_json() -> serde_json::Value {
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "SolinBlog Image Host API",
            "description": "图床功能 API，支持图片上传、查询、更新和删除",
            "version": "1.0.0",
            "contact": {
                "name": "Solin"
            },
            "license": {
                "name": "MIT"
            }
        },
        "servers": [
            {
                "url": "/"
            }
        ],
        "paths": {
            "/api/images": {
                "get": {
                    "tags": ["images"],
                    "summary": "获取图片列表",
                    "description": "分页获取图片列表，支持关键词搜索",
                    "parameters": [
                        {
                            "name": "limit",
                            "in": "query",
                            "description": "每页数量，默认50，范围1-100",
                            "required": false,
                            "schema": {
                                "type": "integer",
                                "minimum": 1,
                                "maximum": 100,
                                "default": 50
                            }
                        },
                        {
                            "name": "offset",
                            "in": "query",
                            "description": "偏移量，默认0",
                            "required": false,
                            "schema": {
                                "type": "integer",
                                "default": 0
                            }
                        },
                        {
                            "name": "q",
                            "in": "query",
                            "description": "搜索关键词",
                            "required": false,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "获取成功",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/ListImagesApiResponse"
                                    }
                                }
                            }
                        },
                        "401": {
                            "$ref": "#/components/responses/ApiError"
                        }
                    },
                    "security": [{"bearer_auth": []}]
                },
                "post": {
                    "tags": ["images"],
                    "summary": "上传图片",
                    "description": "上传新图片，使用 multipart/form-data 格式",
                    "requestBody": {
                        "description": "multipart/form-data，包含 file（图片文件）、alt（可选）、description（可选）",
                        "required": true,
                        "content": {
                            "multipart/form-data": {
                                "schema": {
                                    "type": "object",
                                    "required": ["file"],
                                    "properties": {
                                        "file": {
                                            "type": "string",
                                            "format": "binary",
                                            "description": "图片文件（支持 PNG, JPEG, WEBP, GIF, BMP）"
                                        },
                                        "alt": {
                                            "type": "string",
                                            "description": "图片替代文本，最多200字符",
                                            "maxLength": 200
                                        },
                                        "description": {
                                            "type": "string",
                                            "description": "图片描述，最多1000字符",
                                            "maxLength": 1000
                                        }
                                    }
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "上传成功",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/UploadImageApiResponse"
                                    }
                                }
                            }
                        },
                        "400": {
                            "$ref": "#/components/responses/ApiError"
                        },
                        "401": {
                            "$ref": "#/components/responses/ApiError"
                        },
                        "413": {
                            "$ref": "#/components/responses/ApiError"
                        },
                        "415": {
                            "$ref": "#/components/responses/ApiError"
                        }
                    },
                    "security": [{"bearer_auth": []}]
                }
            },
            "/api/images/{image_id}": {
                "get": {
                    "tags": ["images"],
                    "summary": "获取单个图片信息",
                    "parameters": [
                        {
                            "name": "image_id",
                            "in": "path",
                            "description": "图片ID",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "获取成功",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/ImageMetaApiResponse"
                                    }
                                }
                            }
                        },
                        "401": {
                            "$ref": "#/components/responses/ApiError"
                        },
                        "404": {
                            "$ref": "#/components/responses/ApiError"
                        }
                    },
                    "security": [{"bearer_auth": []}]
                },
                "patch": {
                    "tags": ["images"],
                    "summary": "更新图片元信息",
                    "parameters": [
                        {
                            "name": "image_id",
                            "in": "path",
                            "description": "图片ID",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "requestBody": {
                        "description": "更新的字段（alt 和/或 description）",
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/UpdateImageRequest"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "更新成功",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/ImageMetaApiResponse"
                                    }
                                }
                            }
                        },
                        "400": {
                            "$ref": "#/components/responses/ApiError"
                        },
                        "401": {
                            "$ref": "#/components/responses/ApiError"
                        },
                        "404": {
                            "$ref": "#/components/responses/ApiError"
                        }
                    },
                    "security": [{"bearer_auth": []}]
                },
                "put": {
                    "tags": ["images"],
                    "summary": "替换图片文件",
                    "parameters": [
                        {
                            "name": "image_id",
                            "in": "path",
                            "description": "图片ID",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "requestBody": {
                        "description": "multipart/form-data，包含 file（新图片文件）、alt（可选）、description（可选）",
                        "required": true,
                        "content": {
                            "multipart/form-data": {
                                "schema": {
                                    "type": "object",
                                    "required": ["file"],
                                    "properties": {
                                        "file": {
                                            "type": "string",
                                            "format": "binary",
                                            "description": "新图片文件"
                                        },
                                        "alt": {
                                            "type": "string",
                                            "description": "图片替代文本，最多200字符",
                                            "maxLength": 200
                                        },
                                        "description": {
                                            "type": "string",
                                            "description": "图片描述，最多1000字符",
                                            "maxLength": 1000
                                        }
                                    }
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "替换成功",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/ImageMetaApiResponse"
                                    }
                                }
                            }
                        },
                        "400": {
                            "$ref": "#/components/responses/ApiError"
                        },
                        "401": {
                            "$ref": "#/components/responses/ApiError"
                        },
                        "404": {
                            "$ref": "#/components/responses/ApiError"
                        }
                    },
                    "security": [{"bearer_auth": []}]
                },
                "delete": {
                    "tags": ["images"],
                    "summary": "删除图片",
                    "parameters": [
                        {
                            "name": "image_id",
                            "in": "path",
                            "description": "图片ID",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "删除成功",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/DeleteImageApiResponse"
                                    }
                                }
                            }
                        },
                        "401": {
                            "$ref": "#/components/responses/ApiError"
                        },
                        "404": {
                            "$ref": "#/components/responses/ApiError"
                        }
                    },
                    "security": [{"bearer_auth": []}]
                }
            },
            "/images/{image_id}/{filename}": {
                "get": {
                    "tags": ["images"],
                    "summary": "获取图片文件",
                    "description": "公开访问图片文件，无需认证",
                    "parameters": [
                        {
                            "name": "image_id",
                            "in": "path",
                            "description": "图片ID",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        },
                        {
                            "name": "filename",
                            "in": "path",
                            "description": "文件名",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "图片文件",
                            "content": {
                                "image/png": {},
                                "image/jpeg": {},
                                "image/webp": {},
                                "image/gif": {},
                                "image/bmp": {}
                            }
                        },
                        "304": {
                            "description": "未修改（ETag 匹配）"
                        },
                        "404": {
                            "description": "图片不存在"
                        }
                    }
                }
            }
        },
        "components": {
            "responses": {
                "ApiError": {
                    "description": "请求失败",
                    "content": {
                        "application/json": {
                            "schema": {
                                "$ref": "#/components/schemas/ImageApiErrorResponse"
                            }
                        }
                    }
                }
            },
            "schemas": {
                "ImageApiErrorBody": {
                    "type": "object",
                    "description": "图床 API 错误信息",
                    "required": ["code", "message"],
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "错误代码"
                        },
                        "message": {
                            "type": "string",
                            "description": "错误消息"
                        }
                    }
                },
                "ImageApiErrorResponse": {
                    "type": "object",
                    "description": "图床 API 失败响应",
                    "required": ["success", "data", "error"],
                    "properties": {
                        "success": {
                            "type": "boolean",
                            "const": false
                        },
                        "data": {
                            "type": "null"
                        },
                        "error": {
                            "$ref": "#/components/schemas/ImageApiErrorBody"
                        }
                    }
                },
                "UploadImageApiResponse": {
                    "type": "object",
                    "description": "图床 API 上传成功响应",
                    "required": ["success", "data", "error"],
                    "properties": {
                        "success": {
                            "type": "boolean",
                            "const": true
                        },
                        "data": {
                            "$ref": "#/components/schemas/UploadImageResponse"
                        },
                        "error": {
                            "type": "null"
                        }
                    }
                },
                "ListImagesApiResponse": {
                    "type": "object",
                    "description": "图床 API 图片列表成功响应",
                    "required": ["success", "data", "error"],
                    "properties": {
                        "success": {
                            "type": "boolean",
                            "const": true
                        },
                        "data": {
                            "$ref": "#/components/schemas/ListImagesResponse"
                        },
                        "error": {
                            "type": "null"
                        }
                    }
                },
                "ImageMetaApiResponse": {
                    "type": "object",
                    "description": "图床 API 图片元信息成功响应",
                    "required": ["success", "data", "error"],
                    "properties": {
                        "success": {
                            "type": "boolean",
                            "const": true
                        },
                        "data": {
                            "$ref": "#/components/schemas/ImageMeta"
                        },
                        "error": {
                            "type": "null"
                        }
                    }
                },
                "DeleteImageApiResponse": {
                    "type": "object",
                    "description": "图床 API 删除成功响应",
                    "required": ["success", "data", "error"],
                    "properties": {
                        "success": {
                            "type": "boolean",
                            "const": true
                        },
                        "data": {
                            "$ref": "#/components/schemas/DeleteImageResponse"
                        },
                        "error": {
                            "type": "null"
                        }
                    }
                },
                "ImageMeta": {
                    "type": "object",
                    "description": "图片元信息",
                    "required": ["image_id", "filename", "url", "content_type", "size_bytes", "width", "height", "sha256", "alt", "description", "created_at", "updated_at"],
                    "properties": {
                        "image_id": {
                            "type": "string",
                            "description": "图片唯一ID"
                        },
                        "filename": {
                            "type": "string",
                            "description": "文件名"
                        },
                        "url": {
                            "type": "string",
                            "description": "图片访问URL"
                        },
                        "relative_path": {
                            "type": "string",
                            "description": "相对存储路径"
                        },
                        "content_type": {
                            "type": "string",
                            "description": "MIME类型"
                        },
                        "size_bytes": {
                            "type": "integer",
                            "description": "文件大小（字节）"
                        },
                        "width": {
                            "type": "integer",
                            "description": "图片宽度（像素）"
                        },
                        "height": {
                            "type": "integer",
                            "description": "图片高度（像素）"
                        },
                        "sha256": {
                            "type": "string",
                            "description": "文件SHA256哈希"
                        },
                        "alt": {
                            "type": "string",
                            "description": "替代文本"
                        },
                        "description": {
                            "type": "string",
                            "description": "图片描述"
                        },
                        "created_at": {
                            "type": "integer",
                            "description": "创建时间（Unix时间戳）"
                        },
                        "updated_at": {
                            "type": "integer",
                            "description": "更新时间（Unix时间戳）"
                        }
                    }
                },
                "UploadImageResponse": {
                    "type": "object",
                    "description": "上传响应",
                    "required": ["image_id", "url", "meta"],
                    "properties": {
                        "image_id": {
                            "type": "string"
                        },
                        "url": {
                            "type": "string"
                        },
                        "meta": {
                            "$ref": "#/components/schemas/ImageMeta"
                        }
                    }
                },
                "ListImagesResponse": {
                    "type": "object",
                    "description": "图片列表响应",
                    "required": ["items", "total", "limit", "offset"],
                    "properties": {
                        "items": {
                            "type": "array",
                            "items": {
                                "$ref": "#/components/schemas/ImageMeta"
                            }
                        },
                        "total": {
                            "type": "integer",
                            "description": "总数"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "每页数量"
                        },
                        "offset": {
                            "type": "integer",
                            "description": "偏移量"
                        }
                    }
                },
                "UpdateImageRequest": {
                    "type": "object",
                    "description": "更新图片请求",
                    "properties": {
                        "alt": {
                            "type": "string",
                            "description": "替代文本",
                            "maxLength": 200
                        },
                        "description": {
                            "type": "string",
                            "description": "图片描述",
                            "maxLength": 1000
                        }
                    }
                },
                "DeleteImageResponse": {
                    "type": "object",
                    "description": "删除响应",
                    "required": ["deleted", "image_id"],
                    "properties": {
                        "deleted": {
                            "type": "boolean"
                        },
                        "image_id": {
                            "type": "string"
                        }
                    }
                }
            },
            "securitySchemes": {
                "bearer_auth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "token",
                    "description": "Bearer token 认证"
                }
            }
        },
        "tags": [
            {
                "name": "images",
                "description": "图床图片管理接口"
            }
        ]
    })
}

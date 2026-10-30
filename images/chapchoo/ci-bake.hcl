variable "BINARY_ARTIFACTS" {
  default = "binary-artifacts"
}

variable "WEB_ARTIFACTS" {
  default = "web-artifacts"
}

group "default" {
  targets = ["backend", "general-web", "admin-web"]
}

target "base" {
  context    = "."
  dockerfile = "images/chapchoo/Dockerfile"
  contexts = {
    web-artifacts = WEB_ARTIFACTS
    binary-artifacts = BINARY_ARTIFACTS
  }
}

target "backend" {
  inherits = ["base"]
  target   = "backend"
  tags     = ["chapchoo-package-backend:ci"]
  output   = ["type=docker"]
}

target "general-web" {
  inherits = ["base"]
  target   = "general-web"
  tags     = ["chapchoo-package-general:ci"]
  output   = ["type=docker"]
}

target "admin-web" {
  inherits = ["base"]
  target   = "admin-web"
  tags     = ["chapchoo-package-admin:ci"]
  output   = ["type=docker"]
}

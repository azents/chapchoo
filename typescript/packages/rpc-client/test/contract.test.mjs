import assert from "node:assert/strict";
import { Buffer } from "node:buffer";
import test from "node:test";

import {
  AdminService,
  AuthService,
  GeneralService,
  Grant,
  ListGrantsRequest,
  SetMemberRoleRequest,
  UpdateTeamRequest,
} from "../dist/generated/accounts.js";

test("optional strings preserve omission separately from an empty value", () => {
  const request = UpdateTeamRequest.fromPartial({
    teamId: "team",
    handle: "",
  });
  const encoded = UpdateTeamRequest.encode(request).finish();
  assert.equal(Buffer.from(encoded).toString("hex"), "0a047465616d1a00");
  const decoded = UpdateTeamRequest.decode(encoded);
  assert.equal(decoded.name, undefined);
  assert.equal(decoded.handle, "");
});

test("int64 pagination is exact and retains optional zero", () => {
  const request = ListGrantsRequest.fromPartial({
    offset: 9223372036854775807n,
    limit: 0n,
  });
  const encoded = ListGrantsRequest.encode(request).finish();
  assert.equal(
    Buffer.from(encoded).toString("hex"),
    "08ffffffffffffffff7f1000",
  );
  const decoded = ListGrantsRequest.decode(encoded);
  assert.equal(decoded.offset, 9223372036854775807n);
  assert.equal(decoded.limit, 0n);
  assert.equal(ListGrantsRequest.decode(new Uint8Array()).offset, undefined);
});

test("unknown enum numbers survive decoding for typed ingress validation", () => {
  const request = SetMemberRoleRequest.fromPartial({
    teamId: "team",
    userId: "user",
    role: 12345,
  });
  const decoded = SetMemberRoleRequest.decode(
    SetMemberRoleRequest.encode(request).finish(),
  );
  assert.equal(decoded.role, 12345);
});

test("timestamp seconds and nanoseconds retain full wire precision", () => {
  const grant = Grant.fromPartial({
    issuedAt: {
      seconds: 9007199254740993n,
      nanos: 123456789,
    },
  });
  const decoded = Grant.decode(Grant.encode(grant).finish());
  assert.deepEqual(decoded.issuedAt, grant.issuedAt);
  assert.equal(decoded.expiresAt, undefined);
});

test("service names stay distinct while authentication is shared", () => {
  assert.equal(
    AuthService.passwordLogin.path,
    "/chapchoo.accounts.Auth/PasswordLogin",
  );
  assert.equal(
    GeneralService.listTeams.path,
    "/chapchoo.accounts.General/ListTeams",
  );
  assert.equal(
    AdminService.initialize.path,
    "/chapchoo.accounts.Admin/Initialize",
  );
});

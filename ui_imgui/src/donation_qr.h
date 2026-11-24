#pragma once
#include <d3d11.h>
#include <imgui.h>
namespace vietflow {
// The texture is owned by the UI's D3D device, including minimize/restore.
bool LoadDonationQrTexture(ID3D11Device* device);
void ReleaseDonationQrTexture();
ImTextureID DonationQrTexture();
}

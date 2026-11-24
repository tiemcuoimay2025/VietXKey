#include "donation_qr.h"
#include "donation_qr_data.h"
#include <wincodec.h>
#include <wrl/client.h>
#include <vector>

namespace vietflow {
using Microsoft::WRL::ComPtr;
static ComPtr<ID3D11ShaderResourceView> qr_view;

static bool DecodeAndUpload(ID3D11Device* device) {
    ComPtr<IWICImagingFactory> factory;
    if (FAILED(CoCreateInstance(CLSID_WICImagingFactory, nullptr, CLSCTX_INPROC_SERVER,
                                IID_PPV_ARGS(&factory)))) return false;
    ComPtr<IWICStream> stream;
    if (FAILED(factory->CreateStream(&stream)) ||
        FAILED(stream->InitializeFromMemory(const_cast<BYTE*>(kDonationQrPng),
                                             static_cast<DWORD>(sizeof(kDonationQrPng))))) return false;
    ComPtr<IWICBitmapDecoder> decoder;
    if (FAILED(factory->CreateDecoderFromStream(stream.Get(), nullptr, WICDecodeMetadataCacheOnLoad,
                                                &decoder))) return false;
    ComPtr<IWICBitmapFrameDecode> frame;
    if (FAILED(decoder->GetFrame(0, &frame))) return false;
    UINT width = 0, height = 0;
    if (FAILED(frame->GetSize(&width, &height)) || width != 256 || height != 256) return false;
    ComPtr<IWICFormatConverter> converter;
    if (FAILED(factory->CreateFormatConverter(&converter)) ||
        FAILED(converter->Initialize(frame.Get(), GUID_WICPixelFormat32bppRGBA, WICBitmapDitherTypeNone,
                                      nullptr, 0, WICBitmapPaletteTypeCustom))) return false;
    const UINT pitch = width * 4;
    std::vector<BYTE> pixels(pitch * height);
    if (FAILED(converter->CopyPixels(nullptr, pitch, static_cast<UINT>(pixels.size()), pixels.data()))) return false;
    D3D11_TEXTURE2D_DESC desc = {};
    desc.Width = width; desc.Height = height;
    desc.MipLevels = desc.ArraySize = 1;
    desc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
    desc.SampleDesc.Count = 1;
    desc.Usage = D3D11_USAGE_IMMUTABLE;
    desc.BindFlags = D3D11_BIND_SHADER_RESOURCE;
    D3D11_SUBRESOURCE_DATA data = {pixels.data(), pitch, 0};
    ComPtr<ID3D11Texture2D> texture;
    if (FAILED(device->CreateTexture2D(&desc, &data, &texture))) return false;
    return SUCCEEDED(device->CreateShaderResourceView(texture.Get(), nullptr, &qr_view));
}

bool LoadDonationQrTexture(ID3D11Device* device) {
    qr_view.Reset();
    if (!device) return false;
    const HRESULT init = CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);
    if (FAILED(init) && init != RPC_E_CHANGED_MODE) return false;
    const bool ok = DecodeAndUpload(device);
    if (SUCCEEDED(init)) CoUninitialize();
    return ok;
}
void ReleaseDonationQrTexture() { qr_view.Reset(); }
ImTextureID DonationQrTexture() { return static_cast<ImTextureID>(reinterpret_cast<intptr_t>(qr_view.Get())); }
}

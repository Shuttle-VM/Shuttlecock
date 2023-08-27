package main

import (
	"fmt"

	"github.com/intel/tee-sdk-go/common"
	"github.com/intel/tee-sdk-go/keyexchange"
	"github.com/intel/tee-sdk-go/sgx"
	"github.com/intel/tee-sdk-go/sgx/ecdsa"
)

func main() {
	// Initialize the SGX enclave
	enclave, err := sgx.NewEnclave("./enclave.signed.so")
	if err != nil {
		fmt.Println("Error initializing enclave:", err)
		return
	}

	// Generate a new ECDSA key pair
	privateKey, publicKey, err := ecdsa.GenerateKeyPair()
	if err != nil {
		fmt.Println("Error generating key pair:", err)
		return
	}

	// Store the key pair in an SGX secure data structure
	keyPair := keyexchange.KeyPair{
		PrivateKey: privateKey,
		PublicKey:  publicKey,
	}
	secureKeyPair, err := enclave.NewSecureData(common.SecureDataConfig{
		Type: common.SecureDataConfigTypePlain,
		Size: keyPair.Size(),
	})
	if err != nil {
		fmt.Println("Error creating secure data structure:", err)
		return
	}
	secureKeyPair.Write(keyPair.Bytes())
}

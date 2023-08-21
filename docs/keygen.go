package main

import (
	"fmt"

	"github.com/intel/tee-sdk-go/ecdsa"
)

func main() {
	// Create a new ECDSA key pair
	privateKey, publicKey, err := ecdsa.GenerateKeyPair()
	if err != nil {
		fmt.Println("Error generating key pair:", err)
		return
	}

	// Print the private and public keys
	fmt.Println("Private key:", privateKey.String())
	fmt.Println("Public key:", publicKey.String())
}

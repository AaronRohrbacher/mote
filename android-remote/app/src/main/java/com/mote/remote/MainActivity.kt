package com.mote.remote

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.media.projection.MediaProjectionManager
import android.net.wifi.WifiManager
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.provider.Settings
import android.text.TextUtils
import android.view.View
import android.widget.Button
import android.widget.TextView
import android.widget.Toast
import java.net.Inet4Address
import java.net.NetworkInterface

class MainActivity : Activity() {
    private val SCREEN_CAPTURE_REQUEST = 1001
    private var statusText: TextView? = null
    private var ipText: TextView? = null
    private var touchStatusText: TextView? = null
    private val handler = Handler(Looper.getMainLooper())

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        
        statusText = findViewById(R.id.statusText)
        ipText = findViewById(R.id.ipText)
        touchStatusText = findViewById(R.id.touchStatusText)

        statusText?.text = "Tap Start to begin"
        touchStatusText?.text = "Touch input: checking..."
        ipText?.text = "Connect to: http://<IP>:8080"

        findViewById<Button>(R.id.startButton)?.setOnClickListener {
            requestScreenCapture()
        }

        findViewById<Button>(R.id.accessibilityButton)?.setOnClickListener {
            openAccessibilitySettings()
        }

        handler.postDelayed({ 
            updateTouchStatus()
            handler.post {
                try {
                    val ip = getLocalIpAddress()
                    ipText?.text = "Connect to: http://$ip:8080"
                } catch (e: Exception) {
                    // Ignore
                }
            }
        }, 1000)
    }

    override fun onResume() {
        super.onResume()
        handler.postDelayed({ 
            updateTouchStatus()
        }, 500)
    }

    private fun updateTouchStatus() {
        val enabled = try {
            TouchInputService.instance != null
        } catch (e: Exception) {
            false
        }
        
        if (enabled) {
            touchStatusText?.text = "✓ Touch input ready"
            touchStatusText?.setTextColor(0xFF4CAF50.toInt())
            findViewById<Button>(R.id.accessibilityButton)?.visibility = View.GONE
        } else {
            touchStatusText?.text = "Touch input required"
            touchStatusText?.setTextColor(0xFFFF9800.toInt())
            findViewById<Button>(R.id.accessibilityButton)?.visibility = View.VISIBLE
        }
        
        handler.postDelayed({ updateTouchStatus() }, 1000)
    }
    

    private fun openAccessibilitySettings() {
        try {
            val intent = Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS)
            startActivity(intent)
            Toast.makeText(this, "Enable 'Mote Remote' in the list", Toast.LENGTH_LONG).show()
        } catch (e: Exception) {
            Toast.makeText(this, "Open Settings > Accessibility manually", Toast.LENGTH_LONG).show()
        }
    }
    
    private fun requestScreenCapture() {
        try {
            val projectionManager = getSystemService(Context.MEDIA_PROJECTION_SERVICE) as MediaProjectionManager
            startActivityForResult(projectionManager.createScreenCaptureIntent(), SCREEN_CAPTURE_REQUEST)
        } catch (e: Exception) {
            e.printStackTrace()
            statusText?.text = "Error: ${e.message}"
        }
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)

        if (requestCode == SCREEN_CAPTURE_REQUEST) {
            if (resultCode == RESULT_OK && data != null) {
                try {
                    // Start the screen cast service
                    val serviceIntent = Intent(this, ScreenCastService::class.java).apply {
                        putExtra("resultCode", resultCode)
                        putExtra("data", data)
                    }
                    
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                        startForegroundService(serviceIntent)
                    } else {
                        startService(serviceIntent)
                    }
                    
                    statusText?.text = "Streaming..."
                    
                    // Minimize to background after short delay
                    Handler(Looper.getMainLooper()).postDelayed({
                        moveTaskToBack(true)
                    }, 1000)
                } catch (e: Exception) {
                    e.printStackTrace()
                    statusText?.text = "Error starting service: ${e.message}"
                }
            } else {
                statusText?.text = "Permission denied"
                Toast.makeText(this, "Screen capture permission required", Toast.LENGTH_LONG).show()
            }
        }
    }

    private fun getLocalIpAddress(): String {
        try {
            // Try WifiManager first (may not be available on all AOSP builds)
            try {
                val wifiService = applicationContext.getSystemService(Context.WIFI_SERVICE)
                if (wifiService != null) {
                    val wifiManager = wifiService as? WifiManager
                    val wifiInfo = wifiManager?.connectionInfo
                    val ipInt = wifiInfo?.ipAddress ?: 0
                    if (ipInt != 0) {
                        return String.format(
                            "%d.%d.%d.%d",
                            ipInt and 0xff,
                            ipInt shr 8 and 0xff,
                            ipInt shr 16 and 0xff,
                            ipInt shr 24 and 0xff
                        )
                    }
                }
            } catch (e: Exception) {
                // WiFi service not available or permission denied - continue to fallback
            }
            
            // Fallback: enumerate network interfaces
            try {
                val interfaces = NetworkInterface.getNetworkInterfaces()
                if (interfaces != null) {
                    while (interfaces.hasMoreElements()) {
                        try {
                            val intf = interfaces.nextElement()
                            val addresses = intf.inetAddresses
                            if (addresses != null) {
                                while (addresses.hasMoreElements()) {
                                    val addr = addresses.nextElement()
                                    if (!addr.isLoopbackAddress && addr is Inet4Address) {
                                        val hostAddress = addr.hostAddress
                                        if (hostAddress != null && hostAddress.isNotEmpty()) {
                                            return hostAddress
                                        }
                                    }
                                }
                            }
                        } catch (e: Exception) {
                            // Skip this interface and continue
                            continue
                        }
                    }
                }
            } catch (e: Exception) {
                // Network interface enumeration failed
            }
        } catch (e: Exception) {
            // All methods failed
        }
        return "unknown"
    }

    override fun onDestroy() {
        handler.removeCallbacksAndMessages(null)
        super.onDestroy()
    }
}
